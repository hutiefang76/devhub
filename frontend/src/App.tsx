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

interface PythonEnvironment {
  name: string
  source: string
  path: string
  version: string | null
  is_active: boolean
}

type Language = 'python' | 'java' | 'javascript' | 'rust' | 'go' | 'docker' | 'other'

function App() {
  const [currentLang, setCurrentLang] = useState<Language>('python')

  // Python 状态
  const [pythonInfo, setPythonInfo] = useState<DetectionInfo | null>(null)
  const [pythonEnvs, setPythonEnvs] = useState<PythonEnvironment[]>([])
  const [pythonMirrors, setPythonMirrors] = useState<Mirror[]>([])
  const [currentPythonMirror, setCurrentPythonMirror] = useState<string | null>(null)
  const [pythonSpeedResults, setPythonSpeedResults] = useState<Map<string, number>>(new Map())
  const [pythonTesting, setPythonTesting] = useState(false)
  const [showCreateVenv, setShowCreateVenv] = useState(false)
  const [newVenvName, setNewVenvName] = useState('')
  const [newVenvVersion, setNewVenvVersion] = useState('')

  // JavaScript 状态
  const [jsMirrors, setJsMirrors] = useState<Mirror[]>([])
  const [currentJsMirror, setCurrentJsMirror] = useState<string | null>(null)
  const [jsSpeedResults, setJsSpeedResults] = useState<Map<string, number>>(new Map())
  const [jsTesting, setJsTesting] = useState(false)

  // Rust 状态
  const [rustMirrors, setRustMirrors] = useState<Mirror[]>([])
  const [currentRustMirror, setCurrentRustMirror] = useState<string | null>(null)
  const [rustSpeedResults, setRustSpeedResults] = useState<Map<string, number>>(new Map())
  const [rustTesting, setRustTesting] = useState(false)

  // Java 状态
  const [mavenMirrors, setMavenMirrors] = useState<Mirror[]>([])
  const [currentMavenMirror, setCurrentMavenMirror] = useState<string | null>(null)
  const [gradleMirrors, setGradleMirrors] = useState<Mirror[]>([])
  const [currentGradleMirror, setCurrentGradleMirror] = useState<string | null>(null)
  const [javaSpeedResults, setJavaSpeedResults] = useState<Map<string, number>>(new Map())
  const [javaTesting, setJavaTesting] = useState(false)

  // Go 状态
  const [goMirrors, setGoMirrors] = useState<Mirror[]>([])
  const [currentGoMirror, setCurrentGoMirror] = useState<string | null>(null)
  const [goSpeedResults, setGoSpeedResults] = useState<Map<string, number>>(new Map())
  const [goTesting, setGoTesting] = useState(false)

  // Docker 状态
  const [dockerMirrors, setDockerMirrors] = useState<Mirror[]>([])
  const [currentDockerMirror, setCurrentDockerMirror] = useState<string | null>(null)
  const [dockerSpeedResults, setDockerSpeedResults] = useState<Map<string, number>>(new Map())
  const [dockerTesting, setDockerTesting] = useState(false)

  // 系统工具 状态
  const [gitMirrors, setGitMirrors] = useState<Mirror[]>([])
  const [currentGitMirror, setCurrentGitMirror] = useState<string | null>(null)
  const [homebrewMirrors, setHomebrewMirrors] = useState<Mirror[]>([])
  const [currentHomebrewMirror, setCurrentHomebrewMirror] = useState<string | null>(null)

  const [message, setMessage] = useState('')

  useEffect(() => {
    loadData()
  }, [currentLang])

  const loadData = async () => {
    switch (currentLang) {
      case 'python': await loadPythonData(); break
      case 'javascript': await loadJavaScriptData(); break
      case 'rust': await loadRustData(); break
      case 'java': await loadJavaData(); break
      case 'go': await loadGoData(); break
      case 'docker': await loadDockerData(); break
      case 'other': await loadSystemToolsData(); break
    }
  }

  const loadPythonData = async () => {
    try {
      const info = await invoke<DetectionInfo>('detect_python')
      setPythonInfo(info)

      const envList = await invoke<PythonEnvironment[]>('list_python_environments')
      setPythonEnvs(envList)

      const current = await invoke<string | null>('get_current_pip_mirror')
      setCurrentPythonMirror(current)
      const mirrorList = await invoke<Mirror[]>('list_pip_mirrors')
      setPythonMirrors(mirrorList)
    } catch (error) {
      console.error('加载Python数据失败:', error)
      setMessage(`错误: ${error}`)
    }
  }

  const loadJavaScriptData = async () => {
    try {
      const current = await invoke<string | null>('get_current_npm_mirror')
      setCurrentJsMirror(current || 'https://registry.npmjs.org')
      const mirrorList = await invoke<Mirror[]>('list_npm_mirrors')
      setJsMirrors(mirrorList)
    } catch (error) {
      console.error('加载JavaScript数据失败:', error)
    }
  }

  const loadRustData = async () => {
    try {
      const current = await invoke<string | null>('get_current_cargo_mirror')
      setCurrentRustMirror(current)
      const mirrorList = await invoke<Mirror[]>('list_cargo_mirrors')
      setRustMirrors(mirrorList)
    } catch (error) {
      console.error('加载Rust数据失败:', error)
    }
  }

  const loadJavaData = async () => {
    try {
      const mavenCurrent = await invoke<string | null>('get_current_maven_mirror')
      setCurrentMavenMirror(mavenCurrent)
      const mavenList = await invoke<Mirror[]>('list_maven_mirrors')
      setMavenMirrors(mavenList)

      const gradleCurrent = await invoke<string | null>('get_current_gradle_mirror')
      setCurrentGradleMirror(gradleCurrent)
      const gradleList = await invoke<Mirror[]>('list_gradle_mirrors')
      setGradleMirrors(gradleList)
    } catch (error) {
      console.error('加载Java数据失败:', error)
    }
  }

  const loadGoData = async () => {
    try {
      const current = await invoke<string | null>('get_current_go_mirror')
      setCurrentGoMirror(current)
      const mirrorList = await invoke<Mirror[]>('list_go_mirrors')
      setGoMirrors(mirrorList)
    } catch (error) {
      console.error('加载Go数据失败:', error)
    }
  }

  const loadDockerData = async () => {
    try {
      const current = await invoke<string | null>('get_current_docker_mirror')
      setCurrentDockerMirror(current)
      const mirrorList = await invoke<Mirror[]>('list_docker_mirrors')
      setDockerMirrors(mirrorList)
    } catch (error) {
      console.error('加载Docker数据失败:', error)
    }
  }

  const loadSystemToolsData = async () => {
    try {
      const gitCurrent = await invoke<string | null>('get_current_git_mirror')
      setCurrentGitMirror(gitCurrent)
      const gitList = await invoke<Mirror[]>('list_git_mirrors')
      setGitMirrors(gitList)

      try {
        const homebrewCurrent = await invoke<string | null>('get_current_homebrew_mirror')
        setCurrentHomebrewMirror(homebrewCurrent)
        const homebrewList = await invoke<Mirror[]>('list_homebrew_mirrors')
        setHomebrewMirrors(homebrewList)
      } catch (e) {
        // Ignore if not macOS
      }
    } catch (error) {
      console.error('加载系统工具数据失败:', error)
    }
  }

  const formatLatency = (ms: number): string => {
    if (ms === Number.MAX_VALUE || ms > 10000) return '超时'
    return `${ms}ms`
  }

  const getSortedMirrors = (mirrors: Mirror[], results: Map<string, number>): Mirror[] => {
    return [...mirrors].sort((a, b) => {
      const latencyA = results.get(a.name) ?? Number.MAX_VALUE
      const latencyB = results.get(b.name) ?? Number.MAX_VALUE
      return latencyA - latencyB
    })
  }

  const testSpeed = async (mirrors: Mirror[], setResults: React.Dispatch<React.SetStateAction<Map<string, number>>>) => {
    const results = new Map<string, number>()
    for (const mirror of mirrors) {
      try {
        const latency = await invoke<number>('test_mirror_speed', { url: mirror.url })
        results.set(mirror.name, latency)
      } catch (e) {
        results.set(mirror.name, Number.MAX_VALUE)
      }
    }
    setResults(results)
  }

  const applyMirror = async (mirror: Mirror, command: string) => {
    try {
      await invoke(command, { mirror })
      setMessage(`已切换到 ${mirror.name}`)
      await loadData()
    } catch (error) {
      setMessage(`切换失败: ${error}`)
    }
  }

  const restoreDefault = async (command: string) => {
    try {
      await invoke(command)
      setMessage('已恢复默认配置')
      await loadData()
    } catch (error) {
      setMessage(`恢复失败: ${error}`)
    }
  }

  const switchPythonEnv = async (env: PythonEnvironment) => {
    try {
      await invoke('switch_python_env', { env })
      setMessage(`已切换到 ${env.name}`)
      await loadData()
    } catch (error) {
      setMessage(`切换失败: ${error}`)
    }
  }

  const createVenv = async () => {
    if (!newVenvName || !newVenvVersion) {
      setMessage('请填写虚拟环境名称和Python版本')
      return
    }
    try {
      await invoke('create_venv', {
        name: newVenvName,
        pythonVersion: newVenvVersion,
        path: null
      })
      setMessage(`已创建虚拟环境 ${newVenvName}`)
      setShowCreateVenv(false)
      setNewVenvName('')
      setNewVenvVersion('')
      await loadData()
    } catch (error) {
      setMessage(`创建失败: ${error}`)
    }
  }

  const deleteVenv = async (env: PythonEnvironment) => {
    if (!confirm(`确定要删除虚拟环境 ${env.name} 吗？`)) return
    try {
      await invoke('delete_venv', { env })
      setMessage(`已删除虚拟环境 ${env.name}`)
      await loadData()
    } catch (error) {
      setMessage(`删除失败: ${error}`)
    }
  }

  const renderMirrorGrid = (
    mirrors: Mirror[],
    currentMirror: string | null,
    speedResults: Map<string, number>,
    testing: boolean,
    setTesting: React.Dispatch<React.SetStateAction<boolean>>,
    setResults: React.Dispatch<React.SetStateAction<Map<string, number>>>,
    applyCmd: string,
    restoreCmd?: string
  ) => (
    <div className="mirror-config">
      <div className="section-header">
        <div className="actions">
          <button onClick={() => { setTesting(true); testSpeed(mirrors, setResults).then(() => setTesting(false)); }} disabled={testing}>
            {testing ? '⏳ 测速中...' : '⚡ 测速'}
          </button>
          {restoreCmd && (
            <button onClick={() => restoreDefault(restoreCmd)} className="secondary">
              🔄 恢复默认
            </button>
          )}
        </div>
      </div>

      {currentMirror && (
        <div className="current-mirror">
          <strong>当前镜像源:</strong> {mirrors.find(m => m.url === currentMirror)?.name || currentMirror}
        </div>
      )}

      <div className="mirror-grid">
        {getSortedMirrors(mirrors, speedResults).map((mirror) => {
          const latency = speedResults.get(mirror.name)
          const isCurrent = currentMirror === mirror.url
          const fastestLatency = Math.min(...Array.from(speedResults.values()).filter(v => v < Number.MAX_VALUE))
          const isFastest = latency !== undefined && latency === fastestLatency && latency < Number.MAX_VALUE

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
                onClick={() => applyMirror(mirror, applyCmd)}
                disabled={isCurrent}
                className={isCurrent ? 'applied' : ''}
              >
                {isCurrent ? '✓ 已应用' : '应用'}
              </button>
            </div>
          )
        })}
      </div>
    </div>
  )

  return (
    <div className="container">
      <header>
        <h1>🚀 DevHub Pro</h1>
        <p className="subtitle">优雅轻巧的开发环境管理工具</p>
        <nav className="lang-nav">
          <button className={currentLang === 'python' ? 'active' : ''} onClick={() => setCurrentLang('python')}>Python</button>
          <button className={currentLang === 'java' ? 'active' : ''} onClick={() => setCurrentLang('java')}>Java</button>
          <button className={currentLang === 'javascript' ? 'active' : ''} onClick={() => setCurrentLang('javascript')}>JavaScript</button>
          <button className={currentLang === 'rust' ? 'active' : ''} onClick={() => setCurrentLang('rust')}>Rust</button>
          <button className={currentLang === 'go' ? 'active' : ''} onClick={() => setCurrentLang('go')}>Go</button>
          <button className={currentLang === 'docker' ? 'active' : ''} onClick={() => setCurrentLang('docker')}>Docker</button>
          <button className={currentLang === 'other' ? 'active' : ''} onClick={() => setCurrentLang('other')}>其他</button>
        </nav>
      </header>

      <main>
        {currentLang === 'python' && (
          <>
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
                <h2>Python 环境列表</h2>
                <div className="actions">
                  <button onClick={() => setShowCreateVenv(!showCreateVenv)} className="secondary">
                    {showCreateVenv ? '取消' : '+ 创建虚拟环境'}
                  </button>
                </div>
              </div>

              {showCreateVenv && (
                <div className="create-venv-form">
                  <input
                    type="text"
                    placeholder="环境名称"
                    value={newVenvName}
                    onChange={(e) => setNewVenvName(e.target.value)}
                  />
                  <input
                    type="text"
                    placeholder="Python版本 (如 3.12)"
                    value={newVenvVersion}
                    onChange={(e) => setNewVenvVersion(e.target.value)}
                  />
                  <button onClick={createVenv}>创建</button>
                </div>
              )}

              <div className="env-list">
                {pythonEnvs.map((env) => (
                  <div key={env.name} className={`env-item ${env.is_active ? 'active' : ''}`}>
                    <div className="env-info">
                      <span className="env-name">{env.name}</span>
                      <span className="env-source">{env.source}</span>
                      {env.version && <span className="env-version">v{env.version}</span>}
                    </div>
                    <div className="env-actions">
                      {!env.is_active && (
                        <button onClick={() => switchPythonEnv(env)} className="small">
                          切换
                        </button>
                      )}
                      {(env.source === 'Venv' || env.source === 'CondaEnv') && (
                        <button onClick={() => deleteVenv(env)} className="small danger">
                          删除
                        </button>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </section>

            <section className="mirror-config">
              <h2>pip 镜像源配置</h2>
              {renderMirrorGrid(pythonMirrors, currentPythonMirror, pythonSpeedResults, pythonTesting, setPythonTesting, setPythonSpeedResults, 'apply_pip_mirror', 'restore_pip_default')}
            </section>
          </>
        )}

        {currentLang === 'java' && (
          <>
            <section className="mirror-config">
              <h2>Maven 镜像源配置</h2>
              {renderMirrorGrid(mavenMirrors, currentMavenMirror, javaSpeedResults, javaTesting, setJavaTesting, setJavaSpeedResults, 'apply_maven_mirror')}
            </section>

            <section className="mirror-config">
              <h2>Gradle 镜像源配置</h2>
              {renderMirrorGrid(gradleMirrors, currentGradleMirror, javaSpeedResults, javaTesting, setJavaTesting, setJavaSpeedResults, 'apply_gradle_mirror')}
            </section>
          </>
        )}

        {currentLang === 'javascript' && (
          <section className="mirror-config">
            <h2>npm 镜像源配置</h2>
            {renderMirrorGrid(jsMirrors, currentJsMirror, jsSpeedResults, jsTesting, setJsTesting, setJsSpeedResults, 'apply_npm_mirror', 'restore_npm_default')}
          </section>
        )}

        {currentLang === 'rust' && (
          <section className="mirror-config">
            <h2>Cargo 镜像源配置</h2>
            {renderMirrorGrid(rustMirrors, currentRustMirror, rustSpeedResults, rustTesting, setRustTesting, setRustSpeedResults, 'apply_cargo_mirror', 'restore_cargo_default')}
          </section>
        )}

        {currentLang === 'go' && (
          <section className="mirror-config">
            <h2>Go Modules 镜像源配置</h2>
            {renderMirrorGrid(goMirrors, currentGoMirror, goSpeedResults, goTesting, setGoTesting, setGoSpeedResults, 'apply_go_mirror')}
          </section>
        )}

        {currentLang === 'docker' && (
          <section className="mirror-config">
            <h2>Docker 镜像加速配置</h2>
            {renderMirrorGrid(dockerMirrors, currentDockerMirror, dockerSpeedResults, dockerTesting, setDockerTesting, setDockerSpeedResults, 'apply_docker_mirror')}
            <p className="text-muted" style={{marginTop: '16px'}}>⚠️ 修改后需要重启 Docker 服务</p>
          </section>
        )}

        {currentLang === 'other' && (
          <>
            <section className="mirror-config">
              <h2>Git 镜像源配置</h2>
              {renderMirrorGrid(gitMirrors, currentGitMirror, new Map(), false, () => {}, () => new Map(), 'apply_git_mirror')}
            </section>

            {homebrewMirrors.length > 0 && (
              <section className="mirror-config">
                <h2>Homebrew 镜像源配置</h2>
                {renderMirrorGrid(homebrewMirrors, currentHomebrewMirror, new Map(), false, () => {}, () => new Map(), 'apply_homebrew_mirror')}
              </section>
            )}
          </>
        )}
      </main>

      {message && (
        <div className="message">
          {message}
        </div>
      )}
    </div>
  )
}

export default App
