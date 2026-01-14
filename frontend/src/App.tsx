import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import {
  Layout,
  Menu,
  Card,
  Button,
  Table,
  Tag,
  Space,
  Message,
  Spin,
  Typography,
  Divider,
  Tooltip,
  Select,
  Alert,
  Badge,
} from '@arco-design/web-react'
import {
  IconThunderbolt,
  IconRefresh,
  IconCheck,
  IconSync,
  IconInfoCircle,
  IconFolder,
  IconFile,
  IconCopy,
  IconArrowUp,
  IconExclamationCircle,
} from '@arco-design/web-react/icon'
import '@arco-design/web-react/dist/css/arco.css'
import './App.css'

const { Sider, Content } = Layout
const { Title, Text } = Typography

interface Mirror {
  name: string
  url: string
}

interface ToolStatus {
  name: string
  current_url: string | null
  current_name: string | null
}

interface SpeedTestResult {
  name: string
  url: string
  latency_ms: number
  is_timeout: boolean
}

interface SystemInfo {
  os: string
  os_version: string
  arch: string
}

interface ToolInfo {
  name: string
  installed: boolean
  version: string | null
  install_path: string | null
  config_path: string | null
  supported_on_current_os: boolean
}

interface InstalledVersion {
  version: string
  path: string | null
  is_current: boolean
}

interface VersionManagerInfo {
  manager_name: string
  installed: boolean
  current_version: string | null
  versions: InstalledVersion[]
  env_var_name: string | null
  env_var_value: string | null
  is_consistent: boolean
  inconsistency_message: string | null
}

interface VersionUpdateInfo {
  tool: string
  current_version: string | null
  latest_version: string | null
  has_update: boolean
  update_url: string | null
}

interface InstallSource {
  manager: string
  path: string
}

interface ConflictInfo {
  tool: string
  has_conflict: boolean
  sources: InstallSource[]
  warning_message: string | null
}

type TabKey = 'python' | 'javascript' | 'rust' | 'java' | 'go' | 'docker' | 'system'

const TOOL_MAP: Record<TabKey, string[]> = {
  python: ['pip', 'uv', 'conda'],
  javascript: ['npm', 'yarn', 'pnpm'],
  rust: ['cargo'],
  java: ['maven', 'gradle'],
  go: ['go'],
  docker: ['docker'],
  system: ['brew', 'choco', 'apt', 'git'],
}

const TAB_LABELS: Record<TabKey, string> = {
  python: 'Python',
  javascript: 'JavaScript',
  rust: 'Rust',
  java: 'Java',
  go: 'Go',
  docker: 'Docker',
  system: '系统工具',
}

// 本地存储 key
const STORAGE_KEY = 'devhub_speed_results'

function loadAllSpeedResults(): Record<string, SpeedTestResult[]> {
  try {
    const data = localStorage.getItem(STORAGE_KEY)
    return data ? JSON.parse(data) : {}
  } catch {
    return {}
  }
}

function saveSpeedResults(tool: string, results: SpeedTestResult[]) {
  try {
    const all = loadAllSpeedResults()
    all[tool] = results
    localStorage.setItem(STORAGE_KEY, JSON.stringify(all))
  } catch {
    // ignore
  }
}

function getSpeedResults(tool: string): Map<string, SpeedTestResult> {
  const all = loadAllSpeedResults()
  const results = all[tool] || []
  const map = new Map<string, SpeedTestResult>()
  results.forEach(r => map.set(r.name, r))
  return map
}

function App() {
  const [currentTab, setCurrentTab] = useState<TabKey>('python')
  const [currentTool, setCurrentTool] = useState<string>('pip')
  const [mirrors, setMirrors] = useState<Mirror[]>([])
  const [speedResults, setSpeedResults] = useState<Map<string, SpeedTestResult>>(new Map())
  const [currentStatus, setCurrentStatus] = useState<ToolStatus | null>(null)
  const [testing, setTesting] = useState(false)
  const [applying, setApplying] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null)
  const [toolsInfo, setToolsInfo] = useState<Map<string, ToolInfo>>(new Map())
  const [versionManager, setVersionManager] = useState<VersionManagerInfo | null>(null)
  const [switchingVersion, setSwitchingVersion] = useState(false)
  const [versionUpdate, setVersionUpdate] = useState<VersionUpdateInfo | null>(null)
  const [conflictInfo, setConflictInfo] = useState<ConflictInfo | null>(null)

  // 加载系统信息和工具信息
  useEffect(() => {
    const loadSystemData = async () => {
      try {
        const [sysInfo, allToolsInfo] = await Promise.all([
          invoke<SystemInfo>('get_system_info'),
          invoke<ToolInfo[]>('get_all_tools_info'),
        ])
        setSystemInfo(sysInfo)
        const infoMap = new Map<string, ToolInfo>()
        allToolsInfo.forEach(t => infoMap.set(t.name, t))
        setToolsInfo(infoMap)
      } catch (error) {
        console.error('Failed to load system info:', error)
      }
    }
    loadSystemData()
  }, [])

  useEffect(() => {
    const tools = TOOL_MAP[currentTab]
    if (tools.length > 0) {
      // 找第一个支持的工具
      const supportedTool = tools.find(t => {
        const info = toolsInfo.get(t)
        return !info || info.supported_on_current_os
      })
      setCurrentTool(supportedTool || tools[0])
    }
  }, [currentTab, toolsInfo])

  useEffect(() => {
    loadToolData()
  }, [currentTool])

  const loadToolData = async () => {
    if (!currentTool) return

    const info = toolsInfo.get(currentTool)
    if (info && !info.supported_on_current_os) {
      setMirrors([])
      setCurrentStatus(null)
      setVersionManager(null)
      setVersionUpdate(null)
      setConflictInfo(null)
      return
    }

    setLoading(true)
    const savedResults = getSpeedResults(currentTool)
    setSpeedResults(savedResults)
    try {
      const [mirrorList, status, vmInfo, updateInfo, conflict] = await Promise.all([
        invoke<Mirror[]>('list_mirrors', { name: currentTool }),
        invoke<ToolStatus>('get_tool_status', { name: currentTool }),
        invoke<VersionManagerInfo | null>('get_version_manager_info', { tool: currentTool }),
        invoke<VersionUpdateInfo | null>('check_version_update', { tool: currentTool }),
        invoke<ConflictInfo>('check_tool_conflict', { tool: currentTool }),
      ])
      setMirrors(mirrorList)
      setCurrentStatus(status)
      setVersionManager(vmInfo)
      setVersionUpdate(updateInfo)
      setConflictInfo(conflict)
    } catch (error) {
      Message.error(`加载失败: ${error}`)
    } finally {
      setLoading(false)
    }
  }

  const handleTestSpeed = async () => {
    setTesting(true)
    setSpeedResults(new Map())
    try {
      const results = await invoke<SpeedTestResult[]>('test_mirrors', { name: currentTool })
      const resultMap = new Map<string, SpeedTestResult>()
      results.forEach(r => resultMap.set(r.name, r))
      setSpeedResults(resultMap)
      saveSpeedResults(currentTool, results)
      Message.success('测速完成')
    } catch (error) {
      Message.error(`测速失败: ${error}`)
    } finally {
      setTesting(false)
    }
  }

  const handleApplyMirror = async (mirror: Mirror) => {
    setApplying(mirror.name)
    try {
      await invoke('apply_mirror', { name: currentTool, mirror })
      Message.success(`已切换到 ${mirror.name}`)
      const status = await invoke<ToolStatus>('get_tool_status', { name: currentTool })
      setCurrentStatus(status)
    } catch (error) {
      Message.error(`切换失败: ${error}`)
    } finally {
      setApplying(null)
    }
  }

  const handleRestoreDefault = async () => {
    try {
      await invoke('restore_default', { name: currentTool })
      Message.success('已恢复默认配置')
      const status = await invoke<ToolStatus>('get_tool_status', { name: currentTool })
      setCurrentStatus(status)
    } catch (error) {
      Message.error(`恢复失败: ${error}`)
    }
  }

  const handleApplyFastest = async () => {
    setTesting(true)
    try {
      const fastest = await invoke<Mirror>('apply_fastest_mirror', { name: currentTool })
      Message.success(`已切换到最快镜像: ${fastest.name}`)
      const status = await invoke<ToolStatus>('get_tool_status', { name: currentTool })
      setCurrentStatus(status)
    } catch (error) {
      Message.error(`操作失败: ${error}`)
    } finally {
      setTesting(false)
    }
  }

  const handleSyncJava = async (mirrorName: string) => {
    try {
      await invoke('sync_java_mirrors', { mirrorName })
      Message.success(`Maven 和 Gradle 已同步到 ${mirrorName}`)
      const status = await invoke<ToolStatus>('get_tool_status', { name: currentTool })
      setCurrentStatus(status)
    } catch (error) {
      Message.error(`同步失败: ${error}`)
    }
  }

  const handleSwitchVersion = async (version: string) => {
    setSwitchingVersion(true)
    try {
      await invoke('switch_version', { tool: currentTool, version })
      Message.success(`已切换到版本 ${version}`)
      // 重新加载版本信息
      const vmInfo = await invoke<VersionManagerInfo | null>('get_version_manager_info', { tool: currentTool })
      setVersionManager(vmInfo)
    } catch (error) {
      Message.error(`切换失败: ${error}`)
    } finally {
      setSwitchingVersion(false)
    }
  }

  const handleInstallTool = async (tool: string) => {
    Message.loading({ content: `正在安装 ${tool}...`, duration: 0, id: 'install' })
    try {
      const result = await invoke<string>('install_tool', { name: tool })
      Message.success({ content: result, id: 'install' })
      // 重新加载工具信息
      const allToolsInfo = await invoke<ToolInfo[]>('get_all_tools_info')
      const infoMap = new Map<string, ToolInfo>()
      allToolsInfo.forEach(t => infoMap.set(t.name, t))
      setToolsInfo(infoMap)
      loadToolData()
    } catch (error) {
      Message.error({ content: `安装失败: ${error}`, id: 'install' })
    }
  }

  const handleSyncJavaHome = async () => {
    try {
      await invoke('sync_java_home')
      Message.success('已同步 JAVA_HOME')
      // 重新加载版本信息
      const vmInfo = await invoke<VersionManagerInfo | null>('get_version_manager_info', { tool: currentTool })
      setVersionManager(vmInfo)
    } catch (error) {
      Message.error(`同步失败: ${error}`)
    }
  }

  const getSortedMirrors = () => {
    return [...mirrors].sort((a, b) => {
      const resultA = speedResults.get(a.name)
      const resultB = speedResults.get(b.name)
      if (!resultA && !resultB) return 0
      if (!resultA) return 1
      if (!resultB) return -1
      if (resultA.is_timeout && !resultB.is_timeout) return 1
      if (!resultA.is_timeout && resultB.is_timeout) return -1
      return resultA.latency_ms - resultB.latency_ms
    })
  }

  const getFastestMirror = () => {
    const valid = Array.from(speedResults.values()).filter(r => !r.is_timeout)
    if (valid.length === 0) return null
    return valid.reduce((a, b) => a.latency_ms < b.latency_ms ? a : b)
  }

  const currentToolInfo = toolsInfo.get(currentTool)
  const isToolSupported = !currentToolInfo || currentToolInfo.supported_on_current_os

  const columns = [
    {
      title: '镜像源',
      dataIndex: 'name',
      render: (name: string, record: Mirror) => {
        const isCurrent = currentStatus?.current_url?.replace(/\/$/, '') === record.url.replace(/\/$/, '')
        const result = speedResults.get(name)
        const fastest = getFastestMirror()
        const isFastest = fastest && fastest.name === name && !result?.is_timeout

        return (
          <Space>
            <Text bold={isCurrent}>{name}</Text>
            {isCurrent && <Tag color="arcoblue">当前</Tag>}
            {isFastest && <Tag color="gold">最快</Tag>}
          </Space>
        )
      },
    },
    {
      title: '延迟',
      dataIndex: 'name',
      width: 120,
      render: (name: string) => {
        const result = speedResults.get(name)
        if (!result) return <Text type="secondary">-</Text>
        if (result.is_timeout) return <Tag color="red">超时</Tag>
        const color = result.latency_ms < 200 ? 'green' : result.latency_ms < 500 ? 'orange' : 'red'
        return <Tag color={color}>{result.latency_ms}ms</Tag>
      },
    },
    {
      title: '操作',
      width: 100,
      render: (_: any, record: Mirror) => {
        const isCurrent = currentStatus?.current_url?.replace(/\/$/, '') === record.url.replace(/\/$/, '')
        return (
          <Button
            type={isCurrent ? 'secondary' : 'primary'}
            size="small"
            disabled={isCurrent}
            loading={applying === record.name}
            onClick={() => handleApplyMirror(record)}
            icon={isCurrent ? <IconCheck /> : undefined}
          >
            {isCurrent ? '已应用' : '应用'}
          </Button>
        )
      },
    },
  ]

  const tools = TOOL_MAP[currentTab]

  // 渲染工具选择器
  const renderToolSelector = () => {
    if (tools.length <= 1) return null

    return (
      <div className="tool-selector">
        {tools.map(t => {
          const info = toolsInfo.get(t)
          const supported = !info || info.supported_on_current_os
          const isActive = t === currentTool

          return (
            <Tooltip
              key={t}
              content={!supported ? `${t} 不支持当前系统 (${systemInfo?.os})` : undefined}
              disabled={supported}
            >
              <Button
                type={isActive ? 'primary' : 'secondary'}
                size="small"
                disabled={!supported}
                onClick={() => supported && setCurrentTool(t)}
                className={!supported ? 'tool-disabled' : ''}
              >
                {t.toUpperCase()}
              </Button>
            </Tooltip>
          )
        })}
      </div>
    )
  }

  return (
    <Layout className="app-layout">
      <Sider width={180} className="app-sider">
        <div className="logo">
          <Title heading={5} style={{ margin: 0 }}>DevHub Pro</Title>
        </div>
        <Menu
          selectedKeys={[currentTab]}
          onClickMenuItem={(key) => setCurrentTab(key as TabKey)}
          style={{ width: '100%', background: 'transparent' }}
        >
          {Object.entries(TAB_LABELS).map(([key, label]) => (
            <Menu.Item key={key} style={{ color: 'var(--text-primary)' }}>{label}</Menu.Item>
          ))}
        </Menu>
        {systemInfo && (
          <div className="system-info">
            <Text type="secondary" style={{ fontSize: 11 }}>
              {systemInfo.os === 'macos' ? 'macOS' : systemInfo.os === 'linux' ? 'Linux' : 'Windows'} · {systemInfo.arch}
            </Text>
          </div>
        )}
      </Sider>

      <Content className="app-content">
        <div className="content-header">
          <Space size="large" style={{ width: '100%', justifyContent: 'space-between' }}>
            <Space size="medium">
              <Title heading={4} style={{ margin: 0 }}>{TAB_LABELS[currentTab]} 镜像源配置</Title>
              {renderToolSelector()}
            </Space>
          </Space>
        </div>

        {/* 工具信息卡片 */}
        {currentToolInfo && isToolSupported && (
          <div className="tool-info-bar">
            <Space split={<Divider type="vertical" />}>
              {currentToolInfo.installed ? (
                <>
                  <Space size="mini">
                    <IconInfoCircle style={{ color: 'var(--system-green)' }} />
                    <Text type="secondary">v{currentToolInfo.version}</Text>
                    {versionUpdate?.has_update && (
                      <Tooltip content={`最新版本: ${versionUpdate.latest_version}`}>
                        <Badge count={<IconArrowUp style={{ color: '#fff', fontSize: 10 }} />} dot dotStyle={{ background: 'var(--system-blue)' }}>
                          <Tag color="blue" size="small" style={{ marginLeft: 4, cursor: 'pointer' }} onClick={() => versionUpdate.update_url && window.open(versionUpdate.update_url, '_blank')}>
                            有更新
                          </Tag>
                        </Badge>
                      </Tooltip>
                    )}
                  </Space>
                  {currentToolInfo.install_path && (
                    <Tooltip content="点击复制路径">
                      <div
                        style={{ cursor: 'pointer', display: 'inline-flex', alignItems: 'center', gap: 4 }}
                        onClick={() => {
                          navigator.clipboard.writeText(currentToolInfo.install_path || '')
                          Message.success('已复制安装路径')
                        }}
                      >
                        <IconFolder style={{ color: 'var(--text-secondary)' }} />
                        <Text type="secondary">{currentToolInfo.install_path}</Text>
                        <IconCopy style={{ color: 'var(--text-secondary)', fontSize: 12 }} />
                      </div>
                    </Tooltip>
                  )}
                  {currentToolInfo.config_path && (
                    <Tooltip content="点击复制路径">
                      <div
                        style={{ cursor: 'pointer', display: 'inline-flex', alignItems: 'center', gap: 4 }}
                        onClick={() => {
                          navigator.clipboard.writeText(currentToolInfo.config_path || '')
                          Message.success('已复制配置文件路径')
                        }}
                      >
                        <IconFile style={{ color: 'var(--text-secondary)' }} />
                        <Text type="secondary">{currentToolInfo.config_path}</Text>
                        <IconCopy style={{ color: 'var(--text-secondary)', fontSize: 12 }} />
                      </div>
                    </Tooltip>
                  )}
                </>
              ) : (
                <Space size="mini">
                  <IconInfoCircle style={{ color: 'var(--system-orange)' }} />
                  <Text type="secondary">未安装</Text>
                </Space>
              )}
            </Space>
          </div>
        )}

        {/* 冲突警告 */}
        {conflictInfo?.has_conflict && conflictInfo.warning_message && (
          <Alert
            type="error"
            icon={<IconExclamationCircle />}
            content={
              <Space direction="vertical" size="mini">
                <span>{conflictInfo.warning_message}</span>
                <Text type="secondary" style={{ fontSize: 12 }}>
                  安装来源: {conflictInfo.sources.map(s => `${s.manager} (${s.path})`).join(' / ')}
                </Text>
              </Space>
            }
            closable
            style={{ marginBottom: 12 }}
          />
        )}

        {/* 版本管理区块 */}
        {versionManager && versionManager.installed && (
          <div className="version-manager-bar">
            <Space split={<Divider type="vertical" />}>
              <Space size="mini">
                <Tag color="cyan">{versionManager.manager_name}</Tag>
                <Text type="secondary">当前版本:</Text>
                <Text bold>{versionManager.current_version || '未设置'}</Text>
              </Space>
              {versionManager.versions.length > 0 && (
                <Space size="mini">
                  <Text type="secondary">切换版本:</Text>
                  <Select
                    size="small"
                    style={{ width: 160 }}
                    value={versionManager.current_version || undefined}
                    onChange={handleSwitchVersion}
                    loading={switchingVersion}
                  >
                    {versionManager.versions.map(v => (
                      <Select.Option key={v.version} value={v.version}>
                        {v.version} {v.is_current && '(当前)'}
                      </Select.Option>
                    ))}
                  </Select>
                </Space>
              )}
              {versionManager.env_var_name && (
                <Tooltip content={versionManager.env_var_value || '未设置'}>
                  <Space size="mini" style={{ cursor: 'pointer' }}>
                    <Text type="secondary">{versionManager.env_var_name}:</Text>
                    <Text type="secondary" style={{ maxWidth: 200, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                      {versionManager.env_var_value || '未设置'}
                    </Text>
                  </Space>
                </Tooltip>
              )}
            </Space>
          </div>
        )}

        {/* 配置一致性警告 */}
        {versionManager && !versionManager.is_consistent && versionManager.inconsistency_message && (
          <Alert
            type="warning"
            content={
              <Space>
                <span>{versionManager.inconsistency_message}</span>
                {versionManager.manager_name === 'jenv' && (
                  <Button size="mini" type="outline" onClick={handleSyncJavaHome}>
                    同步 JAVA_HOME
                  </Button>
                )}
              </Space>
            }
            closable
            style={{ marginBottom: 12 }}
          />
        )}

        <Spin loading={loading} style={{ display: 'block' }}>
          {!isToolSupported ? (
            <Card className="mirror-card">
              <div style={{ textAlign: 'center', padding: '40px 0' }}>
                <Text type="secondary">
                  {currentTool} 不支持当前操作系统 ({systemInfo?.os === 'macos' ? 'macOS' : systemInfo?.os})
                </Text>
              </div>
            </Card>
          ) : currentToolInfo && !currentToolInfo.installed ? (
            <Card className="mirror-card">
              <div style={{ textAlign: 'center', padding: '40px 0' }}>
                <Space direction="vertical" size="medium" align="center">
                  <IconInfoCircle style={{ fontSize: 48, color: 'var(--system-orange)' }} />
                  <Text style={{ fontSize: 16 }}>{currentTool} 未安装</Text>
                  <Text type="secondary">安装后可配置镜像源</Text>
                  <Button
                    type="primary"
                    onClick={() => handleInstallTool(currentTool)}
                  >
                    一键安装
                  </Button>
                </Space>
              </div>
            </Card>
          ) : (
            <Card className="mirror-card">
              <div className="card-header">
                <Space>
                  <Text bold>当前镜像源:</Text>
                  {currentStatus?.current_name ? (
                    <Tag color="arcoblue" size="large">{currentStatus.current_name}</Tag>
                  ) : (
                    <Tag size="large">官方默认</Tag>
                  )}
                </Space>
                <Space>
                  <Button
                    type="primary"
                    icon={<IconThunderbolt />}
                    loading={testing}
                    onClick={handleTestSpeed}
                  >
                    测速
                  </Button>
                  <Button
                    type="outline"
                    icon={<IconSync />}
                    loading={testing}
                    onClick={handleApplyFastest}
                  >
                    一键最快
                  </Button>
                  <Button
                    type="secondary"
                    icon={<IconRefresh />}
                    onClick={handleRestoreDefault}
                  >
                    恢复默认
                  </Button>
                </Space>
              </div>

              <Divider style={{ margin: '16px 0' }} />

              <Table
                columns={columns}
                data={getSortedMirrors()}
                rowKey="name"
                pagination={false}
                border={false}
                size="middle"
              />

              {currentTab === 'java' && tools.length > 1 && (
                <>
                  <Divider style={{ margin: '16px 0' }} />
                  <Card title="一键同步 Maven & Gradle" size="small">
                    <Text type="secondary" style={{ marginBottom: 12, display: 'block' }}>
                      选择一个镜像源，同时应用到 Maven 和 Gradle
                    </Text>
                    <Space wrap>
                      {mirrors.slice(0, 4).map(m => (
                        <Button
                          key={m.name}
                          type="outline"
                          onClick={() => handleSyncJava(m.name)}
                        >
                          同步到 {m.name}
                        </Button>
                      ))}
                    </Space>
                  </Card>
                </>
              )}
            </Card>
          )}
        </Spin>
      </Content>
    </Layout>
  )
}

export default App
