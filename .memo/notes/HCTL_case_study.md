# HCTL案例

我觉得关键问题是 \- 1个repo的多个HCTL instance \(比如mac, ubuntu）之间，是否应当协同工作、如何协同工作？mac上做了一个task，要不要ubuntu上接过来做完？我之前希望是这样，但现在不觉得了 \- 因为以前的视角是harness session，那么就会希望ubuntu上的codex可以接过来正在执行的task。但现在 ① 有了participant概念后，引入了远程agency抽象；② 有了UI\-backend binding概念后，引入了多workbench：单control分界后；我的世界观有了变化。

为了说明这个问题，我专门构造了一个场景作为例子，你可以记下来，未来可以作为reference来验证我们的产品/功能设计。

## 场景环境

- github有个repo，叫gh\-jssdk；gitlab有个repo，叫gl\-jstui

- 一台云端主机cloud，安装了3个harness: cloud\_codex, cloud\_claude, cloud\_glm；有一个agency: cloud\_agency；在这个agency上用前边的harness创建participant templates

    - cloud\_tpl\_sde \(基于cloud\_codex\+skills，以下类似\)

    - cloud\_tpl\_sdet \(cloud\_codex\)

    - cloud\_tpl\_pm \(cloud\_claude\)

    - cloud\_tpl\_sre \(cloud\_glm\)

- 在mac上，安装了2个harness: mac\_gemini, mac\_k3；有一个agency: mac\_agency; 并创建了2个participant templates

    - mac\_tpl\_sdet \(mac\_gemini\)

    - mac\_tpl\_ops \(mac\_k3\)

- 在ubuntu上，安装了2个harness: ubuntu\_glm, ubuntu\_grok；有一个agency: ubuntu\_agency; 并创建了2个participant templates

    - ubuntu\_tpl\_sde \(ubuntu\_grok\)

    - ubuntu\_tpl\_pm \(ubuntu\_glm\)\.

## 实践方案 \- mac

- 启动一个HCTL control instance \(mac\_ctl\)，mac\_ctl基于gh\-jssdk创建了两个本地projects \(mac\_jssdk\_01 和 mac\_jssdk\_02\)。

- 关于mac\_jssdk\_01，mac\_ctl通过不同的agency，拿了以下的participants：

    - 通过mac\_agency \(本地\)，拿了3个participant

        1. mac\_ptcp\_jssdk\_01\_01：基于mac\_tpl\_sdet－所以这会有一个mac\_gemini的session，用的是mac本地的worktree\-checkout－worktree位置跟着harness走。

        2. mac\_ptcp\_jssdk\_01\_02：也基于mac\_tpl\_sdet－虽然模板相同，但这是个新的session和新的worktree\-checkout。它和mac\_ptcp\_01可以share一个object database / \.git directory，因为他俩都来自于一个repo。

        3. mac\_ptcp\_jssdk\_01\_03：基于mac\_tpl\_ops－虽然背后是不同的harness，但这个可以和mac\_ptcp\_jssdk\_01/02共用object database, 只是不同的checkout。

    - 通过cloud\_agency \(远程\)，拿了1个participant

        1. mac\_ptcp\_jssdk\_01\_04：基于cloud\_tpl\_sde \- 这个的checkout就在cloud了，和mac不share一个object database。

- 关于mac\_jssdk\_02，mac\_ctl通过不同的agency，拿了以下的participants：

    - 通过mac\_agency \(本地\)，拿了1个participant

        1. mac\_ptcp\_jssdk\_02\_01：基于mac\_tpl\_ops \- 这个的checkout依然在mac本地了，因此虽然和mac和\_jssdk\_01不是一个project，两者还是可以share同一个object database。两种做法都成立：共用同一个object database，或者单独新clone一个；写验收用例时按用例写清测的是哪种。

    - 通过ubuntu\_agency \(远程\)，拿了2个participant

        1. mac\_ptcp\_jssdk\_02\_02：基于ubuntu\_tpl\_sde－这个的checkout在ubuntu上。

        2. mac\_ptcp\_jssdk\_02\_03：基于ubuntu\_tpl\_pm－这个的checkout在ubuntu上，它和mac\_ptcp\_jssdk\_02\_02共享一个object database。

- 既然mac\_ctl已启动，前端方面：

    - 启动一个mac\_cli来连接上这俩projects \(mac\_jssdk\_01 和 mac\_jssdk\_02\)

    - 也启动一个mac\_bench来连接上这俩projects \(mac\_jssdk\_01 和 mac\_jssdk\_02\) \- 同一个后端可以有多个前端与之相连。

## 实践方案 \- cloud

- 启动一个HCTL control instance \(cloud\_ctl\)，cloud\_ctl基于gl\-jstui和gh\-jssdk分别创建了2个本地project \(cloud\_jstui\_01, cloud_jssdk_01\)。

- 关于cloud\_jstui\_01，cloud\_ctl通过不同的agency，拿了以下的participants：

    - 通过ubuntu\_agency \(远程\)，拿了1个participant

        1. cloud\_ptcp\_jstui\_01\_01：基于ubuntu\_tpl\_sde

    - 通过mac\_agency \(远程\)，拿了1个participant

        1. cloud\_ptcp\_jstui\_01\_02：基于mac\_tpl\_ops

- 关于cloud\_jssdk\_01，cloud\_ctl通过不同的agency，拿了以下的participants：
    - 通过cloud_agency \(本地\)，拿了1个participant

      1. cloud\_ptcp\_jssdk\_01\_01：基于cloud\_tpl\_pm

    - 通过mac\_agency \(远程\)，拿了1个participant

      1. cloud\_ptcp\_jssdk\_01\_02：基于mac\_tpl\_ops

- cloud上只在创建project的时候使用了CLI前端cloud\_cli来连接cloud\_ctl，之后没有常驻的本机前端；合入预览、完成Task这类要人拍板的动作，经ubuntu\_bench远程进来。没有前端不等于不用人决策。

## 实践方案 \- ubuntu

- ubuntu是个『瘦客户端』，没有跑ubuntu\_ctl。

- 前端方面，启动了ubuntu\_bench，连上了3个远程的projects：mac\_jssdk\_02、cloud\_jstui\_01、cloud_jssdk_01。这就是『hctl\-control 一份存储有多个前端 \(hctl\-workbench/hctl\-cli, 本地/远程\)』的典型例子了。

## 必然发生的情形

- 同一个模板被两个控制面雇成不同的参与者实例：mac\_tpl\_ops 同时出现在 mac\_jssdk\_01 和 cloud\_jssdk\_01 里，是两个参与者。

- mac\_ctl 和 cloud\_ctl 会同时对 gh\-jssdk 开 PR、同时请求合入 main。

- mac 上 gh\-jssdk 的对象库会被 mac\_ctl 的参与者和 cloud\_ptcp\_jssdk\_01\_02 共用。

- ubuntu 的参与者要推 gh\-jssdk 和 gl\-jstui，凭据在哪台机器、没有时谁中转。

- gl\-jstui 的完整走通要等第二个外部平台适配器。

- 同一个人在 mac\_ctl 和 cloud\_ctl 里是两个 human actor；多控制面不等于多用户，跨控制面的『同一人』归并本轮不做。

- 跨机返工：cloud 上封存的版本被驳回，返工在 cloud 那台机器上做，用原工作树或重新物化，未封存的字节不搬去别的机器；旧版本的评审失效要让远端参与者看得到。

## 综上所述

- 新的架构已经做了大量的dependency/deployment分离，让各种配置分立远程化。

- 核心的安排是agency, ctl, cli/bench

