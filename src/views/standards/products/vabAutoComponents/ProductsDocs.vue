<!--
 * @Author: ${git_name}
 * @Date: 2025-04-18 16:33:00
 * @LastEditors: trexwb
 * @LastEditTime: 2025-10-15 09:53:49
 * @FilePath: /client/books/web/src/views/standards/products/vabAutoComponents/ProductsDocs.vue
 * @Description: 
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved. 
-->
<template>
  <el-dialog v-model="state.dialogVisible" custom-class="products-docs" fullscreen :modal="false" :show-close="false" size="100%">
    <template #header="{ close, titleId, titleClass }">
      <div class="my-header">
        <el-button circle :icon="Back" @click="close" />
        <h2>{{ getTitle() }}</h2>
        <el-button type="danger" @click="download">
          <el-icon class="el-icon--left">
            <download />
          </el-icon>
          {{ pdfDetailForm.download ? `[已购买]` : `[${pdfDetailForm.credit}点积分]` }}下载PDF
        </el-button>
      </div>
    </template>
    <template #default>
      <div class="pdf-wrap">
        <vab-pdf
          ref="pdfRef"
          class="vue-pdf-embed"
          :page="state.pageNum"
          :scale="state.scale"
          :source="pdfDetailForm.path"
          :style="scaleFun"
          @loaded="handlePdfLoaded"
          @progress="handleProgress"
        />
      </div>
      <div class="dialog-footer-btn">
        <el-button type="primary" @click="close">关闭窗口</el-button>
      </div>
    </template>
    <template #footer>
      <div class="pdf-preview">
        <div class="page-tool">
          <div class="page-tool-item">
            <el-button :disabled="state.pageNum === 1" :icon="ArrowLeftBold" type="primary" @click="handleLastPage" />
          </div>
          <div class="page-tool-item">
            <el-button :disabled="state.scale >= state.maxScale" :icon="ZoomIn" type="primary" @click="handlePageZoomIn" />
          </div>
          <div class="page-tool-item">
            <el-button :disabled="!pdfDetailForm.download" :icon="Download" type="primary" @click="download" />
          </div>
          <div class="page-tool-item">
            <el-button :disabled="state.scale <= 1" :icon="ZoomOut" type="primary" @click="handlePageZoomOut" />
          </div>
          <div class="page-tool-item">
            <el-button :disabled="state.pageNum >= state.maxPage" :icon="ArrowRightBold" type="primary" @click="handleNextPage" />
          </div>
        </div>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { Download, Back, ArrowLeftBold, ArrowRightBold, ZoomIn, ZoomOut } from '@element-plus/icons-vue'
// import { VabRoute } from '/@/router/types'
import { useTabsStore } from '/@/store/modules/tabs'
// import { handleActivePath } from '/@/utils/routes'
import { ElMessageBox, ElButton } from 'element-plus'
import VabPdf from '/@/plugins/VabPdf'
import { docsDownload, docsDetail } from '/@/api/products'
import { useUserStore } from '/@/store/modules/user'
const userStore = useUserStore()
const { credit } = storeToRefs(userStore)

const $baseMessage = inject<any>('$baseMessage')

// const route = useRoute()
// const router = useRouter()
const tabsStore = useTabsStore()
const { delVisitedRoute } = tabsStore
/* 核心响应式状态 */
const state = reactive<any>({
  loading: false,
  dialogVisible: false,
  scale: 1,
  maxScale: 4,
  pageNum: 1,
  numPages: 5,
  maxPage: 999,
  loadingProgress: 0,
})
const templateName = 'StandardsProductsAnnotationInterpretation'
defineOptions({
  name: templateName,
})

const pdfDetailForm = ref<any>({})
const pdfRef = ref<InstanceType<typeof VabPdf> | null>(null)

const integral = credit // 账户积分
// const goBack = async () => {
//   await delVisitedRoute(handleActivePath(route as VabRoute, true))
//   if (route.query.pid) {
//     return router.push(`/standards/products?id=${route.query.pid}`)
//   } else {
//     window.close()
//   }
//   // await history.back()
// }

const scaleFun = computed(() => {
  return {
    width: '100vh', //按照pdf大约7：10的宽高比，对应80vh的高度，直接设置高度不生效
    // height: "80vh",
    transform: `scale(${state.scale})`,
    transformOrigin: 'center top', // 设置缩放原点为顶部中心，避免放大时内容被遮挡
  }
})
const handlePdfLoaded = (pdf: any) => {
  state.maxPage = Number(pdf.numPages || 999)
  // console.log('handlePdfLoaded:', pdf.numPages)
}
const handleProgress = ({ loaded, total }: { loaded: number; total: number }) => {
  state.loadingProgress = Math.round((loaded / total) * 100)
}
const handleLastPage = () => {
  if (state.pageNum > 1) {
    state.pageNum -= 1
  }
}
const handleNextPage = () => {
  if (state.pageNum < state.numPages) {
    state.pageNum += 1
  } else {
    if (state.numPages < state.maxPage) {
      ElMessageBox({
        title: '购买提示',
        showCancelButton: true,
        showConfirmButton: true,
        closeOnClickModal: false,
        message: h('p', null, [
          h(
            'p',
            null,
            `需要消耗${pdfDetailForm.value.credit}积分，才能继续查看剩余文档。您当前拥有${integral.value}积分，是否确定使用积分？`
          ),
          h('p', { style: 'color: red' }, '提示：支付后可以下载资料，重复下载不会重复扣分！'),
        ]),
        cancelButtonText: '再想想',
        confirmButtonText: '购买',
        beforeClose: (action, instance, done) => {
          if (action === 'confirm') {
            buyPdf().finally(done)
          } else {
            done()
          }
        },
      })
    } else {
      $baseMessage('已经到最后一页', 'warning', 'hey')
    }
  }
}
const handlePageZoomIn = () => {
  if (state.scale < 4) {
    state.scale += 0.5
  } else {
    $baseMessage('已经放大到最大了', 'warning', 'hey')
  }
}
const handlePageZoomOut = () => {
  if (state.scale > 1) {
    state.scale -= 0.5
  } else {
    $baseMessage('已经缩放到最小了', 'warning', 'hey')
  }
}

const getTitle = () => {
  const title = pdfDetailForm.value.title || ''
  const matchResult = title.match(/^(.*?)\.(pdf|PDF)$/i)

  if (matchResult) {
    return matchResult[1] // 提取文件名部分
  }
  return title
}

const getPdfDetail = async (id: number) => {
  try {
    if (state.loading) return
    state.loading = true
    const { data }: any = await docsDetail({ id })
    pdfDetailForm.value = data
    state.pageNum = 1
    state.numPages = data?.download ? state.maxPage : 5
    state.loading = false
  } catch (error) {
    console.error('Fetch data error:', error)
    // 可以加入错误处理逻辑，如显示错误信息
  }
}
const download = () => {
  if (pdfDetailForm.value.download) {
    downloadPdf()
    return
  }
  if (Number(integral.value) >= Number(pdfDetailForm.value.credit)) {
    ElMessageBox({
      title: '',
      showCancelButton: true,
      showConfirmButton: true,
      message: h('p', null, [
        h('p', null, `下载资料需要消耗${pdfDetailForm.value.credit}积分。您当前拥有${integral.value}积分，是否确定使用积分进行下载？`),
        h('p', { style: 'color: red' }, '提示：重复下载不会重复扣分！'),
      ]),
      cancelButtonText: '取消',
      confirmButtonText: '确定',
      beforeClose: (action, instance, done) => {
        if (action === 'confirm') {
          downloadPdf().finally(done)
        } else {
          done()
        }
      },
    })
  } else {
    ElMessageBox({
      title: '',
      showCancelButton: false,
      showConfirmButton: false,
      message: h('p', null, [
        h('p', null, `下载资料需要消耗${pdfDetailForm.value.credit}积分。您当前积分不足${pdfDetailForm.value.credit}分!`),
        h('p', null, `请联系客服进行充值(13216118255)`),
        // h('a', { href: 'https://example.com', target: '_blank' }, '立刻充值'),
      ]),
    })
  }
}

const buyPdf = async () => {
  try {
    if (!pdfDetailForm.value?.id) {
      ElMessageBox.alert('缺少必要的参数 ID，请重试!', '错误', {
        confirmButtonText: '确定',
      })
      return
    }
    const { data }: any = await docsDownload({ id: pdfDetailForm.value.id })
    // 使用 fetch 请求 PDF 文件内容并转换为 Blob
    if (data.url) {
      pdfDetailForm.value.download = data || null
      state.numPages = state.maxPage
      state.pageNum += 1
    }
  } catch (error) {
    console.error('There was an error downloading the file!', error)
  }
}

const downloadPdf = async () => {
  try {
    if (!pdfDetailForm.value?.id) {
      ElMessageBox.alert('缺少必要的参数 ID，请重试!', '错误', {
        confirmButtonText: '确定',
      })
      return
    }

    const { data }: any = await docsDownload({ id: pdfDetailForm.value.id })
    // 设置下载的文件名
    const fileName = pdfDetailForm.value.title || 'downloaded-file.pdf'

    // 快速下载
    // pdfRef.value?.download(fileName)

    // 使用 fetch 请求 PDF 文件内容并转换为 Blob
    const response = await fetch(data.url)
    if (!response.ok) {
      throw new Error(`Failed to fetch PDF: ${response.statusText}`)
    }
    const blob = await response.blob()
    // 创建一个临时 URL 指向 Blob 数据
    const url = window.URL.createObjectURL(blob)
    // 动态创建一个 HTML <a> 元素，用于触发下载
    const link = document.createElement('a')
    link.href = url
    link.setAttribute('download', fileName)
    // 将链接添加到 DOM 中，并触发点击事件
    document.body.appendChild(link)
    link.click()
    // 清理：移除链接和释放临时 URL
    document.body.removeChild(link)
    window.URL.revokeObjectURL(url)
    // if (!data.url) {
    //   throw new Error('Invalid response: Missing file URL');
    // }
    // // 封装为一个 Blob 对象， 生成一个临时的 URL。可以被浏览器用来访问文件内容，便于后续创建下载链接。
    // // const url = window.URL.createObjectURL(new Blob([data]))
    // // 动态创建一个 HTML a 的元素，触发文件下载
    // const link = document.createElement('a')
    // // 指定 a 元素的href 属性，指定下载文件的来源。
    // link.href = data.url// url
    // // 设置 a 元素的 download 属性，指定下载文件的名称。
    // const fileName = pdfDetailForm.value.title || 'downloaded-file';
    // link.setAttribute('download', encodeURIComponent(fileName)); // 设置下载的文件名
    // document.body.appendChild(link)
    // link.click()
    // link.remove()
    // window.URL.revokeObjectURL(data.url); // 释放临时 URL
  } catch (error) {
    console.error('There was an error downloading the file!', error)
  }
}

const showDetail = async (id: number) => {
  await getPdfDetail(id)
  open()
}

// 对话框控制方法
const open = () => {
  state.dialogVisible = true
}
const close = async () => {
  state.dialogVisible = false
  state.scale = 1
}

// 暴露组件方法
defineExpose({ showDetail })

/* 生命周期钩子 */
onMounted(() => {})
onBeforeUnmount(() => {
  /* 组件卸载前清理逻辑 */
})
onBeforeMount(() => {
  // getPdfDetail()
})
</script>

<style lang="scss">
.my-header {
  position: fixed;
  top: 0;
  left: 0;
  z-index: 1000;
  display: flex;
  flex-direction: row;
  gap: 16px;
  justify-content: space-between;
  width: 100%;
  height: 60px;
  padding: 10px 20px 0;
  overflow: hidden;
  background: #fff;
  box-shadow: 0 4px 4px -2px rgba(134, 134, 134, 0.1);
}

.el-dialog__footer {
  position: fixed;
  bottom: 0;
  left: 0;
  width: 100%;
  height: 80px;
  padding: 0 20px;
  padding-top: 12px;
  background: #fff;
  box-shadow: 0 -5px 5px -5px rgba(0, 0, 0, 0.3);
}

.dialog-footer-btn {
  position: fixed;
  right: 20px;
  bottom: 24px;
  z-index: 1000;
}

.pdf-wrap {
  display: flex;
  justify-content: center;
  min-height: calc(100vh - 140px); /* 改为 min-height，允许内容扩展 */
  padding-top: 60px;
  padding-bottom: 20px; /* 为阴影预留底部空间 */
  overflow: auto; /* 允许滚动 */
  transform: translateX(0) scale(1);

  .vue-pdf-embed {
    box-sizing: border-box;
    width: auto !important;
    margin-bottom: 20px; /* 为阴影预留空间 */
    text-align: center;
    box-shadow:
      0 8px 16px 0 rgba(0, 0, 0, 0.15),
      0 0 0 1px rgba(0, 0, 0, 0.05);
  }

  canvas {
    width: calc((100vh - 210px) * 0.707) !important;
    max-width: none; /* 移除最大宽度限制，允许放大 */
    /* A4 尺寸比例：210mm × 297mm，约等于 0.707 (宽/高) */
    height: calc(100vh - 210px) !important; /* 调整高度计算，预留阴影空间 */
    aspect-ratio: 210 / 297; /* A4 标准比例 */
  }
}

.pdf-preview {
  width: 100%;

  .page-tool {
    position: relative;
    z-index: 100;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40vw;
    max-width: 450px;
    margin: 5px auto;
    color: white;
    background: rgba(230, 233, 239, 1);
    border-radius: 24px;

    .page-tool-item {
      padding: 8px 15px;
      padding-left: 10px;
      cursor: pointer;
    }
  }
}

/* 对话框样式优化 */
.products-docs {
  .el-dialog__body {
    padding: 0;
    overflow: hidden; /* 防止对话框本身滚动 */
  }
}

@media (max-width: 1000px) {
  .pdf-preview {
    .page-tool {
      width: 40vw;
      margin: 5px 30%;
    }
  }
}
</style>
