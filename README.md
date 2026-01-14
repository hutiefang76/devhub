# DevHub - 开发环境镜像源管理工具

一个高性能的开发环境镜像源管理工具，专为中国开发者优化。

## 功能特性

- 🚀 **14+ 工具支持**: pip, npm, cargo, maven, docker 等
- ⚡ **一键测速**: 并发测试所有镜像源，自动选择最快
- 🔄 **一键切换**: 简单命令即可切换镜像源
- 💾 **自动备份**: 修改配置前自动备份，支持一键恢复
- 🎯 **50+ 镜像源**: 阿里云、清华、腾讯云、华为云等

## 快速开始

### 安装

```bash
# 从源码构建
git clone https://github.com/hutiefang76/devhub.git
cd devhub
cargo build --release

# 添加到 PATH
cp target/release/devhub /usr/local/bin/
```

### 基本用法

```bash
# 查看支持的工具
devhub list

# 查看当前配置状态
devhub status

# 测试镜像源速度
devhub test pip

# 应用指定镜像源
devhub use pip Tuna

# 自动选择最快镜像
devhub use pip --fastest

# 恢复默认配置
devhub restore pip
```

## 支持的工具

| 类别 | 工具 | 配置文件 |
|------|------|----------|
| **Python** | pip, uv, conda | pip.conf, uv.toml, .condarc |
| **JavaScript** | npm, yarn, pnpm | .npmrc, .yarnrc |
| **Rust** | cargo | .cargo/config.toml |
| **Go** | go modules | GOPROXY 环境变量 |
| **Java** | maven, gradle | settings.xml, init.gradle |
| **Container** | docker | daemon.json |
| **System** | brew, apt | 环境变量, sources.list |
| **VCS** | git | .gitconfig |

## 镜像源列表

每个工具都内置了多个中国镜像源:

- **阿里云** (Aliyun)
- **清华大学** (Tuna)
- **中科大** (USTC)
- **腾讯云** (Tencent)
- **华为云** (Huawei)
- 等等...

## 示例

### 查看状态
```bash
$ devhub status
--------------------------------------------------------------------------------
工具         当前源 URL                                            状态
--------------------------------------------------------------------------------
pip        https://mirrors.ustc.edu.cn/pypi/simple            [USTC]
npm        https://registry.npmmirror.com/                    [Taobao]
cargo      默认                                                 [官方/默认]
...
```

### 测速
```bash
$ devhub test pip
排名   延迟         名称           URL
----------------------------------------------------------------------
1    185ms      USTC         https://mirrors.ustc.edu.cn/pypi/simple
2    220ms      Tuna         https://pypi.tuna.tsinghua.edu.cn/simple
3    406ms      Tencent      https://mirrors.cloud.tencent.com/pypi/simple
...
推荐: 'USTC' 是最快的镜像源
执行 'devhub use pip USTC' 应用此镜像
```

### 切换镜像
```bash
$ devhub use pip --fastest
正在寻找最快的镜像源...
最快镜像源: USTC (185ms)
正在应用 USTC 镜像...
备份已创建: ~/.config/pip/pip.conf.bak.1705234567
成功! pip 现在使用 USTC 镜像
```

## 配置

可以在 `~/.config/devhub/mirrors.json` 自定义镜像源列表。

## 开发

```bash
# 构建
cargo build

# 测试
cargo test

# 运行
cargo run -- status
```

## 许可证

MIT License

## 作者

**Frank Hu**
- GitHub: https://github.com/hutiefang76
- Gitee: https://gitee.com/hutiefang
- Email: hutiefang@gmail.com
