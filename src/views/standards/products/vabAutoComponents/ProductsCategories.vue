<!--
 * @Author: ${git_name}
 * @Date: 2025-04-09 15:21:57
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-05-16 08:57:32
 * @FilePath: /books/web/src/views/standards/products/vabAutoComponents/ProductsCategories.vue
 * @Description: 
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved. 
-->
<template>
  <div v-loading="state.loading" class="column-table-container no-background-container">
    <vab-card>
      <template #header>
        <div class="card-header">
          <span>产品分类</span>
        </div>
      </template>
      <div class="custom-tree">
        <el-tree
          ref="treeRef"
          accordion
          :data="categoriesRows"
          :default-expanded-keys="expandedKeys"
          empty-text=""
          :highlight-current="true"
          :indent="8"
          node-key="id"
          :props="defaultProps"
          @node-click="handleNodeClick"
        >
          <template #default="{ node, data }">
            <div class="custom-tree-node">
              <!-- <el-popover placement="left" trigger="hover" :offset="8" :show-after="300">
                <template #reference>
                  <el-image v-show="data.ico" :src="`${data.ico}!a50`" style="width: auto; height: 25px" />
                </template>
    <div class="tree-node-popover">
      <el-image :src="`${data.ico}!a50`" style="width: 70px; height: auto" />
    </div>
    </el-popover> -->
              <span class="node-label" :title="node.label">{{ node.label }}</span>
            </div>
          </template>
        </el-tree>
      </div>
    </vab-card>
  </div>
</template>

<script setup lang="ts">
import { useAclStore } from '/@/store/modules/acl'
import { categoriesAll } from '/@/api/products'
import { ElTree, tourStepEmits } from 'element-plus'

/* 组件模板名称 */
const templateName = 'StandardsProductsCategories'

/* 定义组件选项 */
defineOptions({
  name: templateName, // 注册组件名称，用于调试和keep-alive缓存标识
})

const props = defineProps(['categorieResult'])
const emit = defineEmits(['handle-change']) // 定义组件事件

/* 权限和语言相关状态 */
const aclStore = useAclStore()
const { defaultItem } = storeToRefs(aclStore) // 解构多语言列表响应式引用

/* 核心响应式状态 */
const state = reactive<any>({
  loading: false,
  defaultLang: defaultItem.value.languages || 'zh-cn', // 默认语言编码
})

// 默认属性映射
const defaultProps = {
  children: 'children',
  label: 'label',
}

// ----------------------------- 树的引用 -----------------------------
const treeRef = ref<InstanceType<typeof ElTree>>()

// 默认展开的节点
const expandedKeys = ref([])
const targetNodeInfo = ref<any>([])
// 当节点被点击的时候触发
const handleNodeClick = (node: any, treeNode: any) => {
  if (!node.children || node.children.length === 0) {
    const parentIds = []
    let currentNode = treeNode
    while (currentNode) {
      if (currentNode.data && currentNode.data.id) {
        parentIds.unshift(currentNode.data.id)
      }
      currentNode = currentNode.parent
    }
    targetNodeInfo.value = {
      id: node.id,
      parentIds: parentIds,
      names: node.names,
      standards: node.standards || [],
      shapes: node.shapes || [],
    }
    // emit('handle-change', targetNodeInfo.value)
  } else {
    targetNodeInfo.value = {
      id: 0,
      parentIds: [],
      names: null,
      standards: [],
      shapes: [],
    }
  }
}
// 监听时传值， 当两次值不相等时触发
watch(
  targetNodeInfo,
  (newValue, oldValue) => {
    if (JSON.stringify(newValue) != JSON.stringify(oldValue) && newValue.id) {
      emit('handle-change', targetNodeInfo.value)
    }
  },
  { immediate: true }
)

const categoriesRows = ref()
const fetchCategories = async () => {
  state.loading = true
  const { data } = await categoriesAll()
  const list = Array.isArray(data) ? data : []
  // 后端 HTTP 已返回树形结构（节点含 children）；IPC 直连返回扁平数组（含 parentId/parent_id）
  const isTree = list.length > 0 && list.some((item: any) => Array.isArray(item.children))
  if (isTree) {
    // 树结构：递归补 label（el-tree 使用）
    const transform = (options: any[]): any[] =>
      options.map((item: any) => ({
        ...item,
        label: item.names?.[state.defaultLang] || '',
        children: item.children && item.children.length > 0 ? transform(item.children) : undefined,
      }))
    categoriesRows.value = transform(list)
  } else {
    // 扁平数组：按 parent_id/parentId 构建树形结构
    const map: Record<number, any> = {}
    const roots: any[] = []
    list.forEach((item: any) => {
      map[item.id] = { ...item, label: item.names?.[state.defaultLang] || '', children: [] }
    })
    list.forEach((item: any) => {
      const node = map[item.id]
      const pid = item.parent_id ?? item.parentId
      if (pid && map[pid]) {
        map[pid].children.push(node)
      } else {
        roots.push(node)
      }
    })
    categoriesRows.value = roots
  }
  setTimeout(() => {
    state.loading = false
  }, 650)
}
const init = () => {
  fetchCategories()
}

/* 生命周期钩子 */
onMounted(() => {
  init()
})
onBeforeUnmount(() => {
  /* 组件卸载前清理逻辑 */
})
onBeforeMount(() => {})
</script>

<style lang="scss">
.card-header {
  display: flex;
  align-items: center;
  min-height: 22px;

  span {
    font-size: 14px;
    font-weight: bold;
  }
}

.el-tree-node {
  padding: 0;
}

.custom-scrollbar .el-scrollbar__wrap {
  width: 100%;
  overflow-x: hidden !important;
  overflow-y: auto !important;
}

.custom-tree-node {
  display: flex;
  gap: 5px;
  align-items: center;
  font-size: 13px;
}

.tree-node-popover {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 100%;

  .text {
    padding-bottom: 4px;
    font-size: 16px;
    font-weight: bold;
    color: var(--default-color);
  }
}

.custom-tree-node .el-icon {
  font-size: 16px;
  color: var(--default-color);
}

.node-label {
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.custom-tree-node:hover {
  cursor: pointer;
}

.el-tree-node__expand-icon {
  font-size: 14px;

  color: var(--default-color);
}

.el-tree-node__content > .el-tree-node__expand-icon {
  padding: 3px !important;
}

.el-tree--highlight-current .el-tree-node.is-current > .el-tree-node__content {
  // background-color: rgba(45, 105, 105, 0.1);
  color: var(--default-color);
}

.el-tree-node__expand-icon {
  font-size: 20px;
}
</style>
