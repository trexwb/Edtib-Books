// [迁移调整] 统一走 src/bridge 桥接层（Tauri invoke），不再直接访问 window.electronAPI
import { bridge } from '/@/bridge'

// vue文件调用此方法，build后将保留源码，若非必须使用动态导入图片，强烈不推荐使用此方法
export const getImageUrl = (name: string): string => new URL(`../${name}`, import.meta.url).href

function extractPathFromUrl(url: string) {
  // // 使用正则表达式匹配协议和域名部分
  // const match = url.match(/^(https?:)?\/\/[^\/]+(\/.*)$/);
  // if (match) {
  //   // 提取域名之后的部分
  //   return encodeURIComponent(match[2]);
  // }
  // // 如果没有匹配到，返回原始字符串
  return encodeURIComponent(url);
}

export const getImageSrc = async (url: string) => {
  if (import.meta.env.VITE_USER_NODE_ENV === 'development') {
    return url;
  }
  async function getCacheFile() {
    try {
      const localPath = await bridge.cacheFile(`${url}`);
      return localPath || `/files/?path=${extractPathFromUrl(`${url}`)}`;
    } catch (error) {
      return `/files/?path=${extractPathFromUrl(`${url}`)}`;
    }
  }
  return url?.startsWith('data:') ? url : await getCacheFile();
}