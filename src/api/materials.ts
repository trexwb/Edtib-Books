/**
 * @description 材料标准 API（免费内容：注册用户全部可见，无购买 / 订阅语义）
 *
 * 对应服务端 FastenerMaterialsService（FrontStandardsController 免费通道 MATERIALS_FNS）：
 * - list / detail：材料库列表与详情（基本信息 + 化学成分 + 近似对照），access 恒为 free
 * - categories / filterOptions：材料类别导航
 * 服务端返回不含任何价格与门禁字段，前端不得渲染购买入口。
 *
 * @Author: trexwb
 * @Date: 2026-09-11
 */

import requestBridge from '/@/utils/requestBridge'
import request from '/@/utils/request'

/**
 * 材料列表（分页 / 筛选）
 * HTTP: /front/standards/materialsList
 * @param data.filter.category_id 材料类别编号
 * @param data.filter.keyword 关键词（牌号 / ISC / 旧牌号 / 其他名称模糊匹配）
 * @returns { total, list }，list 项含 id/code/isc/alias/old_code/standard/standard_name/property/covers/detail/category/density/access='free'
 */
export async function materialsList(data: {
  filter?: { category_id?: number | null; keyword?: string }
  sort?: string
  page?: number
  pageSize?: number
} = {}) {
  const result = await requestBridge.create({
    http: {
      url: '/front/standards/materialsList',
      data,
    },
  })
  return { data: result }
}

/**
 * 材料详情（基本信息 + 化学成分 + 近似对照）
 * HTTP: /front/standards/materialsDetail
 * @param data.id 材料编号
 * @returns { ...材料主体, chemistries[], equivalents[] }
 */
export async function materialsDetail(data: { id: number }) {
  const result = await requestBridge.create({
    http: {
      url: '/front/standards/materialsDetail',
      data,
    },
  })
  return { data: result }
}

/**
 * 材料类别列表（上架，供左侧导航使用）
 * HTTP: /front/standards/materialsCategories
 * @returns { data: { id, code, names, abbreviation, covers, sort }[] }
 */
export async function materialsCategories() {
  const result = await requestBridge.create({
    http: {
      url: '/front/standards/materialsCategories',
      data: {},
    },
  })
  return { data: result }
}

/**
 * 材料筛选候选（当前与 categories 同源，保留扩展位）
 * HTTP: /front/standards/materialsFilterOptions
 * @returns { categories }
 */
export async function materialsFilterOptions(data: { filter?: Record<string, unknown> } = {}) {
  const result = await requestBridge.create({
    http: {
      url: '/front/standards/materialsFilterOptions',
      data,
    },
  })
  return { data: result }
}

export { requestBridge, request }
