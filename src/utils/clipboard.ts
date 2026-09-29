import { gp } from '/@vab/plugins/vab'

function clipboardSuccess(text: any, text1: any) {
  gp.$baseMessage(`拷贝${text1}成功`, 'success', 'hey')
}

function clipboardError(text: any, text1: any) {
  gp.$baseMessage(`拷贝${text1}失败`, 'error', 'hey')
}

/**
 * @description 复制数据
 * @param text
 */
export default function handleClipboard(text: string, text1: string) {
  const { isSupported, copy } = useClipboard()
  if (!isSupported) {
    usePermission('clipboard-write')
  }
  copy(text)
    .then(() => {
      clipboardSuccess(text, text1)
    })
    .catch(() => {
      clipboardError(text, text1)
    })
}
