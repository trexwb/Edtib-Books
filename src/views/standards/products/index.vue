<!--
 * @Author: ${git_name}
 * @Date: 2025-04-18 16:33:00
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-06-13 15:09:52
 * @FilePath: /books/web/src/views/standards/products/index.vue
 * @Description: 
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved. 
-->
<!-- 产品标准 -->
<template>
  <div class="column-table-container no-background-container">
    <el-row :gutter="15">
      <el-col :lg="5" :md="6" :sm="24" :xl="4" :xs="24">
        <products-categories ref="categoriesRef" :categorie-result="categorieResult" @handle-change="handleCategoriesChange" />
      </el-col>
      <el-col :lg="19" :md="18" :sm="24" style="min-height: calc(100vh - 100px) !important" :xl="20" :xs="24">
        <vab-card style="height: 100%">
          <template #header>
            <products-condition
              ref="conditionRef"
              :categorie-result="categorieResult"
              :shape-result="shapeResult"
              :standard-result="standardResult"
              @handle-change="handleConditionChange"
              @handle-clean="handleConditionClean"
            />
          </template>
          <products-filter ref="filterRef" :categorie-result="categorieResult" @handle-change="handleFilterChange" />
          <products-list ref="listRef" :query-form="queryForm" />
        </vab-card>
      </el-col>
    </el-row>
    <div style="height: 25px"></div>
  </div>
</template>

<script lang="ts" setup>
import { useAclStore } from '/@/store/modules/acl'
import { getStorage } from '/@/utils/storage'

const templateName = 'StandardsProducts'
defineOptions({
  name: templateName,
})

/* 权限和语言相关状态 */
const aclStore = useAclStore()
const { configuration, defaultItem } = storeToRefs(aclStore) // 解构多语言列表响应式引用

/* 核心响应式状态 */
const state = reactive<any>({
  loading: false,
  defaultLang: defaultItem.value.languages || 'zh-cn', // 默认语言编码
  standards: [],
})

const categoriesRef = ref<any>(null)
const conditionRef = ref<any>(null)
const filterRef = ref<any>(null)
const listRef = ref<any>(null)

const queryForm = reactive({
  filter: {
    keywords: '',
    id: null as number | string | null,
    standard_id: null as number | string | null,
    shape_id: null as number | string | null | Array<number>,
    category_id: null as number | string | null,
    code: '',
    status: '',
  },
  page: 1,
  pageSize: 20,
  sort: '+sort',
})

const categorieResult = ref<any>(null)
const standardResult = ref<any>(null)
const shapeResult = ref<any[]>([])

const handleCategoriesChange = (categoriesId: any) => {
  categorieResult.value = categoriesId || null
  standardResult.value = null
  shapeResult.value = []
  queryForm.filter.keywords = ''
  queryForm.filter.standard_id = null
  queryForm.filter.shape_id = null
  queryForm.filter.category_id = categoriesId?.id || null
  queryForm.page = 1
  nextTick(() => {
    filterRef.value.handleChange(standardResult.value, shapeResult.value)
    handleFetchData()
  })
}

const handleConditionChange = (condition: any) => {
  categorieResult.value = condition.categories || null
  // category_id修改
  queryForm.filter.category_id = condition.categories?.id || null
  standardResult.value = condition.standards || null
  queryForm.filter.keywords = ''
  queryForm.filter.standard_id = condition.standards?.id || null
  shapeResult.value = condition.shape || []
  // shape_id修改
  queryForm.filter.shape_id = condition.shape.map((item: any) => item.id) || []
  nextTick(() => {
    filterRef.value.handleChange(standardResult.value, shapeResult.value)
    handleFetchData()
  })
}

const handleConditionClean = () => {
  categorieResult.value = null
  queryForm.filter.category_id = null
  standardResult.value = null
  queryForm.filter.standard_id = null
  shapeResult.value = []
  queryForm.filter.keywords = ''
  queryForm.filter.shape_id = null
  nextTick(() => {
    filterRef.value.handleChange(standardResult.value, shapeResult.value)
    handleFetchData()
  })
}

const handleFilterChange = (keywords: string, standard: any, shape: any) => {
  // console.log('handleFilterChange:', keywords, standard, shape)
  standardResult.value = standard || null
  shapeResult.value = shape || []
  queryForm.filter.keywords = keywords || ''
  if (!!keywords) {
    categorieResult.value = null
    standardResult.value = null
    shapeResult.value = []
    queryForm.filter.standard_id = null
    queryForm.filter.shape_id = null
    queryForm.filter.category_id = null
  } else {
    queryForm.filter.standard_id = standard?.id || null
    queryForm.filter.shape_id = (shape || []).map((item: any) => item.id).flat()
  }
  queryForm.page = 1
  handleFetchData()
}

const handleFetchData = () => {
  listRef.value.fetchData(queryForm)
}
</script>

<style lang="scss">
.no-background-container {
  .top-panel {
    display: inherit !important;
  }

  .el-radio-group,
  .el-form-item__content {
    align-items: flex-start;
  }

  .el-tag {
    &.active {
      color: #fff;
    }

    &.primary {
      color: var(--default-color);
    }
  }

  .select-radio {
    &.el-radio {
      height: 30px;
      margin: 0 0 5px 0;
    }

    .image-slot {
      display: flex;
      align-items: flex-start;
      justify-content: center;
      height: 28px;
      padding-top: 4px;
      line-height: 28px;
    }

    .el-radio__inner {
      display: none;
    }

    .el-radio__label {
      display: flex;
      align-items: center;
    }
  }

  .el-radio__inner {
    display: none;
  }

  .el-radio__label {
    position: relative;
    padding-left: 0;
  }

  .vab-card .el-card__header {
    padding: 10px;
    font-size: 16px;
    background: rgba(5, 41, 101, 0.1);
  }

  .el-card__body {
    padding: 12px;
  }

  .el-descriptions__body .el-descriptions__table .el-descriptions__cell,
  .el-input__inner::placeholder {
    font-size: 13px;
  }

  .el-descriptions__body .el-descriptions__table:not(.is-bordered) .el-descriptions__cell {
    padding-bottom: 0;
  }

  .el-form-item__label {
    padding: 0 12px 0 0;
    font-size: 13px;
  }

  .list-box {
    width: 100%;
    // margin-top: 40px;

    .el-card .el-card__body {
      height: auto;
    }

    .name-text {
      font-size: 20px;
      font-weight: bold;
      color: var(--default-color);
    }
  }
}
</style>
