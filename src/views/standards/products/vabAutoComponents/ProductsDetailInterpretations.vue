<!--
 * @Author: ${git_name}
 * @Date: 2025-04-18 16:33:00
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-05-06 13:37:46
 * @FilePath: /books/web/src/views/standards/products/vabAutoComponents/ProductsDetailInterpretations.vue
 * @Description: 
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved. 
-->
<template>
  <vab-card>
    <template #header>
      <div class="card-header">
        <div>标准解读</div>
        <el-tooltip effect="light" :offset="0" placement="left">
          <template #content>
            <p>标准解读均为我司专业人员对相关标准进行的解读说明，如有侵权或错误！</p>
            <p>请联系：13216118255。</p>
            <p>感谢您的理解与支持！</p>
          </template>
          <el-icon>
            <info-filled />
          </el-icon>
        </el-tooltip>
      </div>
    </template>
    <v-md-preview v-if="!state.loading" ref="preview" :text="mdText" />
    <products-docs ref="docsRef" />
  </vab-card>
</template>

<script setup lang="ts">
import VMdPreview from '@kangc/v-md-editor/lib/preview'
import '@kangc/v-md-editor/lib/style/preview.css'
import githubTheme from '@kangc/v-md-editor/lib/theme/github'
import '@kangc/v-md-editor/lib/theme/style/github.css'
import hljs from 'highlight.js'
import { useAclStore } from '/@/store/modules/acl'
import { reactive, ref, computed } from 'vue'
import { InfoFilled } from '@element-plus/icons-vue'

VMdPreview.use(githubTheme, {
  Hljs: hljs,
})

// 定义 props 类型
const props = defineProps(['content'])

// 定义 state 类型
const aclStore = useAclStore()
const state = reactive({
  loading: false,
  defaultLang: computed(() => aclStore.defaultItem.languages || 'zh-cn'),
})

const preview = ref<InstanceType<typeof VMdPreview>>()
const docsRef = ref<any>(null)
const title = ref('')
const mdText = ref('')

const interpretationsChange = () => {
  title.value = props.content?.titles ? props.content?.titles[state.defaultLang] : ''
  const detail = props.content?.detail ? props.content?.detail[state.defaultLang] : ''
  const contentKeywords = props.content?.keywords ? props.content?.keywords[state.defaultLang] : []
  // 初始化 keywordMap
  const keywordMap = new Map<string, string>()
  ;(contentKeywords || []).forEach((item: any) => {
    if (!keywordMap.has(item.keyword)) {
      keywordMap.set(item.keyword, item.id)
    }
  })

  // 提取公共函数：替换关键字
  const replaceKeywords = (detail: string, keywordMap: Map<string, string>): string => {
    let replacedDetail = detail
    keywordMap.forEach((id, keyword) => {
      // 转义关键词中的特殊字符
      const escapedKeyword = keyword.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
      // 构造正则表达式，确保匹配完整的关键词
      const regex = new RegExp(`(?<!\\w)${escapedKeyword}(?!\\w)`, 'g') // 使用负向前瞻和后瞻确保完整匹配
      // 或者更简单的边界处理（假设关键词用空格、标点等分隔）
      // replacedDetail = replacedDetail.replace(regex, `[${keyword}](${process.env.NODE_ENV == 'development' ? '/front/' : '/'}standards/annotationInterpretation?id=${id}&pid=${props.content?.product?.id || ''})`)
      replacedDetail = replacedDetail.replace(
        regex,
        `[${keyword}](${process.env.NODE_ENV == 'development' ? '/front/' : '/'}standards/annotationInterpretation?id=${id})`
      )
    })
    return replacedDetail
  }

  function replaceIdLinks(input: string) {
    const ID_LINK_REGEX = /\[(.*?)\]\(id=(\d+)\)/g
    return input.replace(ID_LINK_REGEX, `[$1](${process.env.NODE_ENV == 'development' ? '/front/' : '/'}standards/products?id=$2)`)
  }
  // 替换 detail 中的关键字
  mdText.value = replaceKeywords(replaceIdLinks(detail), keywordMap)
}

/**
 * 显示渲染图入口方法
 * 组合执行直径判断和渲染图加载流程
 */
const handelDefault = () => {
  state.loading = true
  interpretationsChange()
  state.loading = false
  if (preview.value?.$el) {
    // 获取预览内容的 DOM 节点
    const previewElement = preview.value.$el
    // 监听点击事件
    previewElement.addEventListener('click', (event: any) => {
      // 检查点击的目标是否是 <a> 标签
      if (event.target.tagName === 'A') {
        const href = event.target.getAttribute('href') // 获取链接地址
        // 判断是否包含指定路径
        if (href && href.includes(`${process.env.NODE_ENV == 'development' ? '/front/' : '/'}standards/annotationInterpretation?id=`)) {
          event.preventDefault() // 阻止默认跳转行为
          // 提取 id 参数
          const idMatch = href.match(/id=(\d+)/)
          if (idMatch) {
            // const id = idMatch[1]; // 获取 id 值
            docsRef.value.showDetail(Number(idMatch[1] || 0))
          }
        }
      }
    })
  }
}

// 暴露组件方法给父组件
defineExpose({ handelDefault })

/* 生命周期钩子 */
onMounted(() => {})
onBeforeUnmount(() => {})
onBeforeMount(() => {})
</script>

<style lang="scss" scoped>
.card-header {
  font-size: 14px;
  font-weight: bold;
  color: var(--default-color);
}

.content {
  a {
    font-weight: bold;
    color: var(--default-color);
    text-decoration: underline;
  }

  a[target='_blank'] {
    font-weight: normal;
    color: #000;
    text-decoration: none;
  }
}
</style>
