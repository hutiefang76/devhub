import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import {
  Layout,
  Menu,
  Card,
  Button,
  Table,
  Tag,
  Space,
  Message,
  Radio,
  Spin,
  Typography,
  Divider,
} from '@arco-design/web-react'
import {
  IconThunderbolt,
  IconRefresh,
  IconCheck,
  IconSync,
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

type TabKey = 'python' | 'javascript' | 'rust' | 'java' | 'go' | 'docker' | 'system'

const TOOL_MAP: Record<TabKey, string[]> = {
  python: ['pip', 'uv', 'conda'],
  javascript: ['npm', 'yarn', 'pnpm'],
  rust: ['cargo'],
  java: ['maven', 'gradle'],
  go: ['go'],
  docker: ['docker'],
  system: ['brew', 'apt', 'git'],
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

function App() {
  const [currentTab, setCurrentTab] = useState<TabKey>('python')
  const [currentTool, setCurrentTool] = useState<string>('pip')
  const [mirrors, setMirrors] = useState<Mirror[]>([])
  const [speedResults, setSpeedResults] = useState<Map<string, SpeedTestResult>>(new Map())
  const [currentStatus, setCurrentStatus] = useState<ToolStatus | null>(null)
  const [testing, setTesting] = useState(false)
  const [applying, setApplying] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    const tools = TOOL_MAP[currentTab]
    if (tools.length > 0) {
      setCurrentTool(tools[0])
    }
  }, [currentTab])

  useEffect(() => {
    loadToolData()
  }, [currentTool])

  const loadToolData = async () => {
    if (!currentTool) return
    setLoading(true)
    setSpeedResults(new Map())
    try {
      const [mirrorList, status] = await Promise.all([
        invoke<Mirror[]>('list_mirrors', { name: currentTool }),
        invoke<ToolStatus>('get_tool_status', { name: currentTool }),
      ])
      setMirrors(mirrorList)
      setCurrentStatus(status)
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
      await loadToolData()
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
      await loadToolData()
    } catch (error) {
      Message.error(`恢复失败: ${error}`)
    }
  }

  const handleApplyFastest = async () => {
    setTesting(true)
    try {
      const fastest = await invoke<Mirror>('apply_fastest_mirror', { name: currentTool })
      Message.success(`已切换到最快镜像: ${fastest.name}`)
      await loadToolData()
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
      await loadToolData()
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

  return (
    <Layout className="app-layout">
      <Sider width={180} className="app-sider">
        <div className="logo">
          <Title heading={5} style={{ margin: 0, color: '#fff' }}>DevHub Pro</Title>
        </div>
        <Menu
          selectedKeys={[currentTab]}
          onClickMenuItem={(key) => setCurrentTab(key as TabKey)}
          style={{ width: '100%' }}
        >
          {Object.entries(TAB_LABELS).map(([key, label]) => (
            <Menu.Item key={key}>{label}</Menu.Item>
          ))}
        </Menu>
      </Sider>

      <Content className="app-content">
        <div className="content-header">
          <Space size="large">
            <Title heading={4} style={{ margin: 0 }}>{TAB_LABELS[currentTab]} 镜像源配置</Title>
            {tools.length > 1 && (
              <Radio.Group
                type="button"
                value={currentTool}
                onChange={(value) => setCurrentTool(value)}
                options={tools.map(t => ({ label: t.toUpperCase(), value: t }))}
              />
            )}
          </Space>
        </div>

        <Spin loading={loading} style={{ display: 'block' }}>
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
        </Spin>
      </Content>
    </Layout>
  )
}

export default App
