# cyberlith

adding to app.js unloads the memory enough to start app again?

```
function unload() {
    if (wasm == undefined || wasm == null) {
        console.log("can't unload");
        return;
    }
    console.log("unloading");
    console.log(wasm);
    wasm = undefined;
}

export { initSync, unload }
```