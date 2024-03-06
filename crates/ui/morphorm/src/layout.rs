use smallvec::SmallVec;

use crate::{CacheExt, Cache, LayoutType, Node, NodeExt, percentage_calc, PositionType, Size, Solid, MarginUnits, SizeUnits};

const DEFAULT_MIN: f32 = -f32::MAX;
const DEFAULT_MAX: f32 = f32::MAX;

/// Represents the type of a stretch item. Either space-before, size (main/cross), or space-after.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ItemType {
    Before,
    After,
}

/// Represents a space or size which has stretch units.
#[derive(Copy, Clone)]
struct StretchItem {
    // The child index of the item.
    index: usize,
    // The stretch factor of the item.
    factor: f32,
    // The type of stretch item, either space-before, size, or space-after.
    item_type: ItemType,
    // The computed size of the stretch item.
    computed: f32,
    // Whether or not the stretch item is frozen.
    frozen: bool,
}

impl StretchItem {
    pub fn new(index: usize, factor: f32, item_type: ItemType) -> Self {
        Self { index, factor, item_type, computed: 0.0, frozen: false }
    }
}

#[derive(Debug, Copy, Clone)]
struct ChildNode<'a, N: Node> {
    // A reference to the node.
    node: &'a N,
    // Computed cross-before space of the node.
    cross_before: f32,
    // Computed cross size of the node.
    cross: f32,
    // Computed cross-after space of the node.
    cross_after: f32,
    // Computed main-before space of the node.
    main_before: f32,
    // Computed main size of the node.
    main: f32,
    // Computed main-after space of the node.
    main_after: f32,
}

/// Performs layout on the given node returning its computed size.
///
/// The algorithm recurses down the tree, in depth-first order, and performs
/// layout on every node starting from the input `node`.
///
/// # Arguments
///
/// * `node` - Root node to start layout from.
/// * `parent_layout_type` - The [`LayoutType`] of the parent of the `node`.
/// * `parent_main` - The size of the parent of the `node` on its main axis. If the `node` has no parent then pass the size of the node.
/// * `cross_size` - The size of the `node` along its cross axis.
/// * `cache` - A mutable reference to the [`Cache`].
/// * `tree` - A mutable reference to the [`Tree`](crate::Node::Tree).
/// * `store` - A mutable reference to the [`Store`](crate::Node::Store).
/// * `sublayout` - A mutable reference to the [`SubLayout`](crate::Node::SubLayout) context.
///
/// # Example
///
/// ```
/// layout(&root, LayoutType::Column, 600.0, 600.0, &mut cache, &tree, &store, &mut sublayout);
/// ```
#[allow(clippy::too_many_arguments)]
pub(crate) fn layout<N, C>(
    node: &N,
    parent_layout_type: LayoutType,
    parent_main: f32,
    cross_size: f32,
    cache: &mut C,
    tree: &<N as Node>::Tree,
    store: &<N as Node>::Store,
) -> Size
where
    N: Node,
    C: Cache<Node = N>,
{
    let parent_padding_main: f32 = match node.padding_main_before(store, parent_layout_type) {
        SizeUnits::Pixels(val) => val,
        SizeUnits::Percentage(val) => percentage_calc(val, parent_main, 0.0),
        _ => 0.0,
    } + match node.padding_main_after(store, parent_layout_type) {
        SizeUnits::Pixels(val) => val,
        SizeUnits::Percentage(val) => percentage_calc(val, parent_main, 0.0),
        _ => 0.0,
    };
    let parent_padding_cross: f32 = match node.padding_cross_before(store, parent_layout_type) {
        SizeUnits::Pixels(val) => val,
        SizeUnits::Percentage(val) => percentage_calc(val, cross_size, 0.0),
        _ => 0.0,
    } + match node.padding_cross_after(store, parent_layout_type) {
        SizeUnits::Pixels(val) => val,
        SizeUnits::Percentage(val) => percentage_calc(val, cross_size, 0.0),
        _ => 0.0,
    };

    // The layout type of the node. Determines the main and cross axes of the children.
    let layout_type = node.layout_type(store).unwrap_or_default();

    // The desired main-axis and cross-axis sizes of the node.
    let main = node.main(store, parent_layout_type);
    let cross = node.cross(store, parent_layout_type);

    let main_min = node.main_min(store, parent_layout_type).to_px(parent_main, parent_padding_main, DEFAULT_MIN);
    let main_max = node.main_max(store, parent_layout_type).to_px(parent_main, parent_padding_main, DEFAULT_MAX);

    // TODO: Need parent_cross to compute this correctly
    let cross_min = node.cross_min(store, parent_layout_type).to_px(cross_size, parent_padding_cross, DEFAULT_MIN);
    let cross_max = node.cross_max(store, parent_layout_type).to_px(cross_size, parent_padding_cross, DEFAULT_MAX);

    // Compute main-axis size.
    let mut computed_main = match main {
        SizeUnits::Pixels(val) => val,
        SizeUnits::Percentage(val) => percentage_calc(val, parent_main, parent_padding_main).round(),
        SizeUnits::Auto => 0.0,
    };

    // Cross size is determined by the parent.
    let mut computed_cross = cross_size;

    let node_children = node.children(tree).filter(|child| child.visible(store));

    // Get the total number of children of the node.
    let num_children = node_children.count();

    // Get the total number of relative-typed children of the node.
    let num_relative_children = node
        .children(tree)
        .filter(|child| child.position_type(store).unwrap_or_default() == PositionType::Relative)
        .filter(|child| child.visible(store))
        .count();

    // Apply main-axis size constraints for pixels and percentage.
    computed_main = computed_main.min(main_max).max(main_min);

    // TODO: Figure out how to constrain content size on cross axis.
    apply_solid_layout(node, store, &mut computed_main, &mut computed_cross);

    // Return early if there's no children to layout.
    if num_children == 0 {
        return Size { main: computed_main, cross: computed_cross };
    }

    // Determine the parent_main/cross size to pass to the children based on the layout type of the parent and the node.
    // i.e. if the parent layout type and the node layout type are different, swap the main and the cross axes.
    let (mut parent_main, mut parent_cross) = if parent_layout_type == layout_type {
        (computed_main, computed_cross)
    } else {
        (computed_cross, computed_main)
    };

    // Sum of all space and size flex factors on the main-axis of the node.
    let mut main_flex_sum = 0.0;

    // Sum of all child nodes on the main-axis.
    let mut children_main_sum = 0.0;

    // Maximum of all child nodes on the cross-axis.
    let mut children_cross_max = 0.0f32;

    // List of child nodes for the current node.
    let mut children = SmallVec::<[ChildNode<N>; 32]>::with_capacity(num_children);

    // List of stretch nodes for the current node.
    // A stretch node is any flexible space/size. e.g. main_before, and main_after are separate stretch nodes
    let mut main_axis_stretch_nodes = SmallVec::<[StretchItem; 32]>::new();

    let node_child_main_between = node.main_between(store, layout_type);

    // Determine index of first and last relative child nodes.
    let mut iter = node
        .children(tree)
        .filter(|child| child.visible(store))
        .filter(|child| child.position_type(store).unwrap_or_default() == PositionType::Relative)
        .enumerate();

    let first = iter.next().map(|(index, _)| index);
    let last = iter.last().map_or(first, |(index, _)| Some(index));

    let mut node_children = node
        .children(tree)
        .filter(|child| child.visible(store))
        .filter(|child| child.position_type(store).unwrap_or_default() == PositionType::Relative)
        .enumerate()
        .peekable();

    // Compute space and size of non-flexible parent-directed children.
    while let Some((index, child)) = node_children.next() {
        // Get desired space and size.
        let child_margin_main_before = child.margin_main_before(store, layout_type);
        let mut child_margin_main_after = child.margin_main_after(store, layout_type);

        let child_margin_cross_before = child.margin_cross_before(store, layout_type);
        let child_cross = child.cross(store, layout_type);
        let child_margin_cross_after = child.margin_cross_after(store, layout_type);

        // Get fixed-size space and size constraints.
        let child_cross_min = child.cross_min(store, layout_type);
        let child_cross_max = child.cross_max(store, layout_type);


        if last != Some(index) {
            if let Some((_, _)) = node_children.peek() {
                child_margin_main_after.add_size_units(node_child_main_between);
            }
        }

        // Collect stretch main items.
        if let MarginUnits::Stretch(factor) = child_margin_main_before {
            main_flex_sum += factor;
            main_axis_stretch_nodes.push(StretchItem::new(
                index,
                factor,
                ItemType::Before,
            ));
        }

        if let MarginUnits::Stretch(factor) = child_margin_main_after {
            main_flex_sum += factor;
            main_axis_stretch_nodes.push(StretchItem::new(
                index,
                factor,
                ItemType::After,
            ));
        }

        // Compute fixed-size child cross_before.
        let computed_child_cross_before =
            child_margin_cross_before.to_px(parent_cross, parent_padding_cross, 0.0);

        // Compute fixed-size child_cross.
        let mut computed_child_cross = child_cross.to_px_clamped(parent_cross, parent_padding_cross, 0.0, child_cross_min, child_cross_max);

        // Compute fixed-size child cross_after.
        let computed_child_cross_after =
            child_margin_cross_after.to_px(parent_cross, parent_padding_cross, 0.0);

        // Compute fixed-size child main_before.
        let computed_child_main_before =
            child_margin_main_before.to_px(parent_main, parent_padding_main, 0.0);

        // Compute fixed-size child main_after.
        let computed_child_main_after =
            child_margin_main_after.to_px(parent_main, parent_padding_main, 0.0);

        let computed_child_main;
        // Compute fixed-size child main.
        {
            let child_size =
                layout(child, layout_type, parent_main, computed_child_cross, cache, tree, store);

            computed_child_main = child_size.main;
            computed_child_cross = child_size.cross;
        }

        children_main_sum += computed_child_main + computed_child_main_before + computed_child_main_after;
        children_cross_max = children_cross_max.max(computed_child_cross_before + computed_child_cross + computed_child_cross_after);

        children.push(ChildNode {
            node: child,
            cross_before: computed_child_cross_before,
            cross: computed_child_cross,
            cross_after: computed_child_cross_after,
            main_before: computed_child_main_before,
            main: computed_child_main,
            main_after: computed_child_main_after,
        });
    }

    // Determine cross-size of auto node from children.
    if num_relative_children != 0 && node.cross(store, layout_type) == SizeUnits::Auto {
        parent_cross = children_cross_max.min(cross_max).max(cross_min);
    }

    // Compute flexible space and size on the cross-axis for parent-directed children.
    for (index, child) in children
        .iter_mut()
        .filter(|child| child.node.position_type(store).unwrap_or_default() == PositionType::Relative)
        .filter(|child| !child.node.cross(store, layout_type).is_auto())
        .enumerate()
    {
        let child_margin_cross_before = child.node.margin_cross_before(store, layout_type);
        let child_margin_cross_after = child.node.margin_cross_after(store, layout_type);

        let mut cross_flex_sum = 0.0;

        // Collect stretch cross items.
        let mut cross_axis_stretch_nodes = SmallVec::<[StretchItem; 3]>::new();
        if let MarginUnits::Stretch(factor) = child_margin_cross_before {

            cross_flex_sum += factor;

            child.cross_before = 0.0;

            cross_axis_stretch_nodes.push(StretchItem::new(
                index,
                factor,
                ItemType::Before,
            ));
        }

        if let MarginUnits::Stretch(factor) = child_margin_cross_after {

            cross_flex_sum += factor;

            child.cross_after = 0.0;

            cross_axis_stretch_nodes.push(StretchItem::new(
                index,
                factor,
                ItemType::After,
            ));
        }

        loop {
            // If all stretch items are frozen, exit the loop.
            if cross_axis_stretch_nodes.iter().all(|item| item.frozen) {
                break;
            }

            // Compute free space in the cross axis.
            let child_cross_free_space = parent_cross
                - child.cross_before
                - child.cross
                - child.cross_after;

            for item in cross_axis_stretch_nodes.iter_mut().filter(|item| !item.frozen) {
                let actual_cross = (item.factor * child_cross_free_space / cross_flex_sum).round();
                item.computed = actual_cross;
            }

            for item in cross_axis_stretch_nodes.iter_mut().filter(|item| !item.frozen) {
                // Freeze over-stretched items.
                item.frozen = true; // TODO: maybe this should be false

                // If the item is frozen, adjust the used_space and sum of cross stretch factors.
                if item.frozen {
                    cross_flex_sum -= item.factor;

                    match item.item_type {
                        ItemType::Before => {
                            child.cross_before = item.computed;
                        }

                        ItemType::After => {
                            child.cross_after = item.computed;
                        }
                    }
                }
            }
        }

        children_cross_max = children_cross_max.max(child.cross_before + child.cross + child.cross_after);
    }

    // Determine main-size of auto node from children.
    if num_relative_children != 0 && node.main(store, layout_type) == SizeUnits::Auto {
        parent_main = parent_main.max(children_main_sum).min(main_max).max(main_min);
    }

    // Compute flexible space and size on the main axis for parent-directed children.
    if !main_axis_stretch_nodes.is_empty() {
        loop {
            // If all stretch items are frozen, exit the loop.
            if main_axis_stretch_nodes.iter().all(|item| item.frozen) {
                break;
            }

            // Calculate free space on the main-axis.
            let free_main_space = parent_main - children_main_sum;

            for item in main_axis_stretch_nodes.iter_mut().filter(|item| !item.frozen) {
                let actual_main = (item.factor * free_main_space / main_flex_sum).round();

                item.computed = actual_main;
            }

            for item in main_axis_stretch_nodes.iter_mut().filter(|item| !item.frozen) {
                let child = &mut children[item.index];

                // Freeze over-stretched items.
                item.frozen = true; // TODO: maybe this should be false?

                // If the item is frozen, adjust the used_space and sum of cross stretch factors.
                if item.frozen {
                    main_flex_sum -= item.factor;
                    children_main_sum += item.computed;

                    match item.item_type {
                        ItemType::Before => {
                            child.main_before = item.computed;
                        }

                        ItemType::After => {
                            child.main_after = item.computed;
                        }
                    }
                }
            }
        }
    }

    // Compute stretch cross_before and stretch cross_after for auto cross children.
    // TODO: I think this only needs to be done for relative-positioned children...
    for (index, child) in children.iter_mut().filter(|child| child.node.cross(store, layout_type).is_auto()).enumerate()
    {
        let child_margin_cross_before = child.node.margin_cross_before(store, layout_type);
        let child_margin_cross_after = child.node.margin_cross_after(store, layout_type);

        let mut cross_flex_sum = 0.0;

        // Collect stretch cross items.
        let mut cross_axis_stretch_nodes = SmallVec::<[StretchItem; 3]>::new();
        if let MarginUnits::Stretch(factor) = child_margin_cross_before {

            cross_flex_sum += factor;

            child.cross_before = 0.0;

            cross_axis_stretch_nodes.push(StretchItem::new(
                index,
                factor,
                ItemType::Before,
            ));
        }

        if let MarginUnits::Stretch(factor) = child_margin_cross_after {

            cross_flex_sum += factor;

            child.cross_after = 0.0;

            cross_axis_stretch_nodes.push(StretchItem::new(
                index,
                factor,
                ItemType::After,
            ));
        }

        let child_position_type = child.node.position_type(store).unwrap_or_default();

        loop {
            // If all stretch items are frozen, exit the loop.
            if cross_axis_stretch_nodes.iter().all(|item| item.frozen) {
                break;
            }

            // Compute free space in the cross axis.
            let child_cross_free_space = parent_cross
                - child.cross_before
                - child.cross
                - child.cross_after;

            for item in cross_axis_stretch_nodes.iter_mut().filter(|item| !item.frozen) {
                let actual_cross = (item.factor * child_cross_free_space / cross_flex_sum).round();

                item.computed = actual_cross;
            }

            for item in cross_axis_stretch_nodes.iter_mut().filter(|item| !item.frozen) {
                // Freeze over-stretched items.
                item.frozen = true; // TODO: maybe this should be false

                // If the item is frozen, adjust the used_space and sum of cross stretch factors.
                if item.frozen {
                    cross_flex_sum -= item.factor;

                    match item.item_type {
                        ItemType::Before => {
                            child.cross_before = item.computed;
                        }

                        ItemType::After => {
                            child.cross_after = item.computed;
                        }
                    }
                }
            }
        }

        if child_position_type == PositionType::Relative {
            children_cross_max = children_cross_max.max(child.cross_before + child.cross + child.cross_after);
        }
    }

    let node_children = node
        .children(tree)
        .filter(|child| child.position_type(store).unwrap_or_default() == PositionType::Absolute)
        .filter(|child| child.visible(store));

    // Compute space and size of non-flexible self-directed children.
    for child in node_children {
        // Get desired space and size.
        let child_margin_main_before = child.margin_main_before(store, layout_type);
        let child_margin_main_after = child.margin_main_after(store, layout_type);

        let child_margin_cross_before = child.margin_cross_before(store, layout_type);
        let child_cross = child.cross(store, layout_type);
        let child_margin_cross_after = child.margin_cross_after(store, layout_type);

        // Get fixed-size space and size constraints.
        let child_cross_min = child.cross_min(store, layout_type);
        let child_cross_max = child.cross_max(store, layout_type);

        // Compute fixed-size child cross_before.
        let computed_child_cross_before =
            child_margin_cross_before.to_px(parent_cross, parent_padding_cross, 0.0);

        // Compute fixed-size child_cross.
        let mut computed_child_cross = child_cross.to_px_clamped(parent_cross, parent_padding_cross, 0.0, child_cross_min, child_cross_max);

        // Compute fixed-size child cross_after.
        let computed_child_cross_after =
            child_margin_cross_after.to_px(parent_cross, parent_padding_cross, 0.0);

        // Compute fixed-size child main_before.
        let computed_child_main_before =
            child_margin_main_before.to_px(parent_main, parent_padding_main, 0.0);

        // Compute fixed-size child main_after.
        let computed_child_main_after =
            child_margin_main_after.to_px(parent_main, parent_padding_main, 0.0);

        let computed_child_main;

        // Compute fixed-size child main.
        {
            let child_size =
                layout(child, layout_type, parent_main, computed_child_cross, cache, tree, store);

            computed_child_main = child_size.main;
            computed_child_cross = child_size.cross;
        }

        children.push(ChildNode {
            node: child,
            cross_before: computed_child_cross_before,
            cross: computed_child_cross,
            cross_after: computed_child_cross_after,
            main_before: computed_child_main_before,
            main: computed_child_main,
            main_after: computed_child_main_after,
        });
    }

    // Compute flexible space and size on the cross-axis for absolute-typed nodes.
    for (index, child) in children
        .iter_mut()
        .filter(|child| child.node.position_type(store).unwrap_or_default() == PositionType::Absolute)
        .enumerate()
    {
        let child_margin_cross_before = child.node.margin_cross_before(store, layout_type);
        let child_margin_cross_after = child.node.margin_cross_after(store, layout_type);

        let mut cross_flex_sum = 0.0;

        // Collect stretch cross items.
        let mut cross_axis_stretch_nodes = SmallVec::<[StretchItem; 3]>::new();
        if let MarginUnits::Stretch(factor) = child_margin_cross_before {

            cross_flex_sum += factor;

            child.cross_before = 0.0;

            cross_axis_stretch_nodes.push(StretchItem::new(
                index,
                factor,
                ItemType::Before,
            ));
        }

        if let MarginUnits::Stretch(factor) = child_margin_cross_after {

            cross_flex_sum += factor;

            child.cross_after = 0.0;

            cross_axis_stretch_nodes.push(StretchItem::new(
                index,
                factor,
                ItemType::After,
            ));
        }

        let child_position_type = child.node.position_type(store).unwrap_or_default();

        loop {
            // If all stretch items are frozen, exit the loop.
            if cross_axis_stretch_nodes.iter().all(|item| item.frozen) {
                break;
            }

            // Compute free space in the cross axis.
            let child_cross_free_space = parent_cross
                - child.cross_before
                - child.cross
                - child.cross_after;

            for item in cross_axis_stretch_nodes.iter_mut().filter(|item| !item.frozen) {
                let actual_cross = (item.factor * child_cross_free_space / cross_flex_sum).round();

                item.computed = actual_cross;
            }

            for item in cross_axis_stretch_nodes.iter_mut().filter(|item| !item.frozen) {
                // Freeze over-stretched items.
                item.frozen = true; // TODO: maybe this should be false

                // If the item is frozen, adjust the used_space and sum of cross stretch factors.
                if item.frozen {
                    cross_flex_sum -= item.factor;

                    match item.item_type {
                        ItemType::Before => {
                            child.cross_before = item.computed;
                        }

                        ItemType::After => {
                            child.cross_after = item.computed;
                        }
                    }
                }
            }
        }

        if child_position_type == PositionType::Relative {
            children_cross_max = children_cross_max.max(child.cross_before + child.cross + child.cross_after);
        }
    }

    // Compute flexible space and size on the main-axis for absolute nodes.
    for (index, child) in children
        .iter_mut()
        .filter(|child| child.node.position_type(store).unwrap_or_default() == PositionType::Absolute)
        .enumerate()
    {
        let child_main_before = child.node.margin_main_before(store, layout_type);
        let child_main_after = child.node.margin_main_after(store, layout_type);

        let mut child_main_flex_sum = 0.0;

        // Collect stretch main items.
        let mut main_axis_stretch_nodes_2 = SmallVec::<[StretchItem; 3]>::new();
        if let MarginUnits::Stretch(factor) = child_main_before {

            child_main_flex_sum += factor;

            main_axis_stretch_nodes_2.push(StretchItem::new(
                index,
                factor,
                ItemType::Before,
            ));
        }
        if let MarginUnits::Stretch(factor) = child_main_after {

            child_main_flex_sum += factor;

            main_axis_stretch_nodes_2.push(StretchItem::new(
                index,
                factor,
                ItemType::After,
            ));
        }

        loop {
            // If all stretch items are frozen, exit the loop.
            if main_axis_stretch_nodes_2.iter().all(|item| item.frozen) {
                break;
            }

            // Compute free space in the main axis.
            let child_main_free_space = parent_main
                - child.main_before
                - child.main
                - child.main_after;

            // Total size violation in the main axis.

            for item in main_axis_stretch_nodes_2.iter_mut().filter(|item| !item.frozen) {
                let actual_main = (item.factor * child_main_free_space / child_main_flex_sum).round();

                item.computed = actual_main;
            }

            for item in main_axis_stretch_nodes_2.iter_mut().filter(|item| !item.frozen) {
                // Freeze over-stretched items.
                item.frozen = true; // TODO: maybe this should be false

                // If the item is frozen, adjust the used_space and sum of main stretch factors.
                if item.frozen {
                    child_main_flex_sum -= item.factor;

                    match item.item_type {
                        ItemType::Before => {
                            child.main_before = item.computed;
                        }
                        ItemType::After => {
                            child.main_after = item.computed;
                        }
                    }
                }
            }
        }
    }

    // Set size and position of children in the cache.
    let mut main_pos = 0.0;
    for child in children.iter() {
        let child_position_type = child.node.position_type(store).unwrap_or_default();
        match child_position_type {
            PositionType::Absolute => {
                cache.set_rect(
                    child.node,
                    layout_type,
                    child.main_before,
                    child.cross_before,
                    child.main,
                    child.cross,
                );
            }

            PositionType::Relative => {
                main_pos += child.main_before;
                cache.set_rect(
                    child.node,
                    layout_type,
                    main_pos,
                    child.cross_before,
                    child.main,
                    child.cross,
                );
                main_pos += child.main + child.main_after;
            }
        };
    }

    // Determine auto main and cross size from space and size of children.
    if num_relative_children != 0 {

        if parent_layout_type != layout_type {
            std::mem::swap(&mut children_main_sum, &mut children_cross_max)
        };

        if main == SizeUnits::Auto {
            computed_main = children_main_sum.min(main_max).max(main_min);
        }

        if cross == SizeUnits::Auto {
            computed_cross = children_cross_max.min(cross_max).max(cross_min);
        }
    }

    //apply_solid_layout(node, store, &mut computed_main, &mut computed_cross);

    // Return the computed size, propagating it back up the tree.
    Size { main: computed_main, cross: computed_cross }
}

fn apply_solid_layout<N: Node>(node: &N, store: &N::Store, main: &mut f32, cross: &mut f32) {
    // Apply solid layout stuff
    let node_solid = node.solid(store);
    if node_solid.is_some() {
        let aspect_ratio = node.aspect_ratio(store).expect("Solid nodes must have an aspect ratio");
        let node_solid = node_solid.unwrap();
        let computed_main = *cross / aspect_ratio;
        let computed_cross = *main * aspect_ratio;
        match node_solid {
            Solid::Fit => {
                // max axis uses (aspect ratio * min axis)
                *main = computed_main.min(*main);
                *cross = computed_cross.min(*cross);
            }
            Solid::Fill => {
                // min axis uses (aspect ratio * max axis)
                *main = computed_main.max(*main);
                *cross = computed_cross.max(*cross);
            }
        }
    }
}
