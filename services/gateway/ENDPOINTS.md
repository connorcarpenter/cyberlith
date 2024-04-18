endpoints in gateway:

finished:

gateway:443	POST 	/session_connect 	    -> region:14198	    POST	/session/connect
gateway:443	POST 	/user_register 		    -> auth:14206 		POST	/user_register
gateway:443	POST 	/user_register_confirm 	-> auth:14206 		POST	/user_register_confirm
gateway:443	POST 	/user_login 		    -> auth:14206 		POST	/user_login
gateway:443	POST 	/refresh_token_grant 	-> auth:14206 		POST	/refresh_token_grant
gateway:443	POST 	/user_name_forgot 	    -> auth:14206 		POST	/user_name_forgot
gateway:443	POST 	/user_password_forgot 	-> auth:14206 		POST	/user_password_forgot
gateway:443	POST 	/user_password_reset 	-> auth:14206 		POST	/user_password_reset

gateway:443 GET	    /			            -> content:14197    GET     /launcher.html
gateway:443 GET	    /launcher.js	        -> content:14197    GET     /launcher.js
gateway:443 GET	    /launcher_bg.wasm       -> content:14197    GET     /launcher_bg.wasm
gateway:443 GET	    /game		            -> content:14197    GET     /game.html
gateway:443 GET	    /game.js	            -> content:14197    GET     /game.js
gateway:443 GET	    /game_bg.wasm           -> content:14197    GET     /game_bg.wasm

todo:

gateway:443 POST    /session_rtc            -> session:14200    POST    /session_rtc
gateway:443 POST    /world_rtc              -> world:14203      POST    /world_rtc

gateway:80  GET	    *			            <- redirect to gateway:443

