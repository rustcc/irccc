irccc
=====

An irc service for rustcc by rust.

初步设计：

1. 第一次登录，需要输入用户名（昵称）；
2. 一旦输入，不可更改。下次进来时，默认直接就进入聊天室了（放cookie或localstorage中吧）；
3. 进入聊天室界面，开始时，只有一个公共的开放的聊天室，一切从简吧，也看不到当前在线的用户列表；
4. 不能实现单聊；
5. 需要显示当前有多少用户在线；
6. 用户关闭后，需要及时地进行状态的变化和通知；
7. 前端兼容性不考虑ie，也不要用原生写，还是用jquery吧，其它更高级的mvc框架不考虑了；
8. 通信协议可以考虑websocket，如果比较难实现，可以使用ajax long poll。

参考项目：
- https://github.com/cyderize/rust-websocket
- https://github.com/hyperium/hyper
- https://github.com/iron/iron
