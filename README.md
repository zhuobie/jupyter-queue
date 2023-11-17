# 简介

一个简单的jupyter用户排队执行程序，在指定的时间启用指定的用户，同时禁用其他用户，为了释放资源，会同时重启docker容器。

# 使用

包含两个配置文件`config.toml`和`queue.csv`，前者定义了jupyter容器运行的服务器，后者定义了排队规则。当`person`指定为`__ALL`时，代表所有的用户都被启用，没有限制。

将以上配置文件放在可执行文件的相同目录下，运行可执行文件（可以后台执行），输出日志：

```
user1: Docker restart at 2023-11-17 14:44:01.
user1: Disable all users at 2023-11-17 14:44:02.
user1: Enable user user1 at 2023-11-17 14:44:02.
user2: Docker restart at 2023-11-17 14:45:01.
user2: Disable all users at 2023-11-17 14:45:02.
user2: Enable user user2 at 2023-11-17 14:45:02.
__ALL: Docker restart at 2023-11-17 14:46:01.
__ALL: Enable all users at 2023-11-17 14:46:02.
user3: Docker restart at 2023-11-17 14:47:01.
user3: Disable all users at 2023-11-17 14:47:02.
user3: Enable user user3 at 2023-11-17 14:47:02.
__ALL: Docker restart at 2023-11-17 14:48:01.
__ALL: Enable all users at 2023-11-17 14:48:02.
user4: Docker restart at 2023-11-17 14:49:01.
user4: Disable all users at 2023-11-17 14:49:01.
user4: Enable user user4 at 2023-11-17 14:49:02.
__END: Enable all users at 2023-11-17 14:50:00.
```