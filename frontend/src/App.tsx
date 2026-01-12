import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import './App.css'

interface DetectionInfo {
  installed: boolean
  version: string | null
  path: string | null
}

interface Mirror {
  name: string
  url: string
}

function App() {
  const [pythonInfo, setPythonInfo] = useState<DetectionInfo | null>(null)
  const [currentMirror, setCurrentMirror] = useState<string | null>(null)
  const [mirrors, setMirrors] = useState<Mirror[]>([])
  const [speedResults, setSpeedResults] = useState<Map<string, number>>(new Map())
  const [testing, setTesting] = useState(false)
  const [message, setMessage] = useState('')

  useEffect(() => {
    loadData()
  }, [])

  const loadData = async () => {
    try {
      const info = await invoke<DetectionInfo>('detect_python')
      setPythonInfo(info)

      const current = await invoke<string | null>('get_current_pip_mirror')
      setCurrentMirror(current)

      const mirrorList = await invoke<Mirror[]>('list_pip_mirrors')
      setMirrors(mirrorList)
    } catch (error) {
      console.error('加载数据失败:', error)
      setMessage(`错误: ${error}`)
    }
  }

  const testSpeed = async () => {
    setTesting(true)
    setMessage('正在测速...')
    const results = new Map<string, number>()

    try {
      for (const mirror of mirrors) {
        const latency = await invoke<number>('test_mirror_speed', { url: mirror.url })
        results.set(mirror.name, latency)
        setSpeedResults(new Map(results))
      }
      setMessage('测速完成!')
    } catch (error) {
      setMessage(`测速失败: ${error}`)
    } finally {
      setTesting(false)
    }
  }

  const applyMirror = async (mirror: Mirror) => {
    try {
      await invoke('apply_pip_mirror', { mirror })
      setCurrentMirror(mirror.url)
      setMessage(`已切换到 ${mirror.name}`)
    } catch (error) {
      setMessage(`切换失败: ${error}`)
    }
  }

  const restoreDefault = async () => {
    try {
      await invoke('restore_pip_default')
      setCurrentMirror(null)
      setMessage('已恢复默认配置')
    } catch (error) {
      setMessage(`恢复失败: ${error}`)
    }
  }

  const formatLatency = (ms: number): string => {
    if (ms === Number.MAX_VALUE || ms > 10000) {
      return '超时'
    }
    return `${ms}ms`
  }

  const getSortedMirrors = (): Mirror[] => {
    return [...mirrors].sort((a, b) => {
      const latencyA = speedResults.get(a.name) ?? Number.MAX_VALUE
      const latencyB = speedResults.get(b.name) ?? Number.MAX_VALUE
      return latencyA - latencyB
    })
  }

  return (
    <div className="container">
      <header>
        <h1>🚀 DevHub Pro</h1>
        <p className="subtitle">优雅轻巧的开发环境管理工具</p>
      </header>

      <section className="python-status">
        <h2>Python 环境</h2>
        <div className="status-card">
          <div className="status-item">
            <span className="label">状态:</span>
            <span className={pythonInfo?.installed ? 'badge success' : 'badge error'}>
              {pythonInfo?.installed ? '✅ 已安装' : '❌ 未安装'}
            </span>
          </div>
          {pythonInfo?.version && (
            <div className="status-item">
              <span className="label">版本:</span>
              <span className="value">{pythonInfo.version}</span>
            </div>
          )}
          {pythonInfo?.path && (
            <div className="status-item">
              <span className="label">路径:</span>
              <span className="value path">{pythonInfo.path}</span>
            </div>
          )}
        </div>
      </section>

      <section className="mirror-config">
        <div className="section-header">
          <h2>pip 镜像源配置</h2>
          <div className="actions">
            <button onClick={testSpeed} disabled={testing}>
              {testing ? '⏳ 测速中...' : '⚡ 测速'}
            </button>
            <button onClick={restoreDefault} className="secondary">
              🔄 恢复默认
            </button>
          </div>
        </div>

        {currentMirror && (
          <div className="current-mirror">
            <strong>当前镜像源:</strong> {currentMirror}
          </div>
        )}

        <div className="mirror-grid">
          {getSortedMirrors().map((mirror) => {
            const latency = speedResults.get(mirror.name)
            const isCurrent = currentMirror === mirror.url
            const isFastest = latency && latency < Number.MAX_VALUE &&
                             latency === Math.min(...Array.from(speedResults.values()).filter(v => v < Number.MAX_VALUE))

            return (
              <div
                key={mirror.name}
                className={`mirror-card ${isCurrent ? 'current' : ''} ${isFastest ? 'fastest' : ''}`}
              >
                <div className="mirror-info">
                  <h3>{mirror.name}</h3>
                  <p className="mirror-url">{mirror.url}</p>
                  {latency !== undefined && (
                    <div className="latency">
                      延迟: <strong>{formatLatency(latency)}</strong>
                    </div>
                  )}
                  {isFastest && latency && latency < Number.MAX_VALUE && (
                    <span className="badge fastest-badge">🏆 最快</span>
                  )}
                </div>
                <button
                  onClick={() => applyMirror(mirror)}
                  disabled={isCurrent}
                  className={isCurrent ? 'applied' : ''}
                >
                  {isCurrent ? '✓ 已应用' : '应用'}
                </button>
              </div>
            )
          })}
        </div>
      </section>

      {message && (
        <div className="message">
          {message}
        </div>
      )}
    </div>
  )
}

export default App
