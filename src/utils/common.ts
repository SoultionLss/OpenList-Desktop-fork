import { WebviewWindow } from '@tauri-apps/api/webviewWindow'

export const createNewWindow = async (url: string, id: string, title: string) => {
  const webview = new WebviewWindow(id, {
    url,
    title,
    width: 1200,
    height: 800,
    resizable: true,
  })

  webview.once('tauri://created', function () {
    console.log('窗口创建成功！')
  })

  webview.once('tauri://error', function (e) {
    console.error('窗口创建失败:', e)
  })
}
