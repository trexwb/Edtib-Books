/**
 * @description 产品标准 API（离线优先）
 * 优先使用 IPC 直连本地数据库，失败时 fallback 到 HTTP
 * @Author: trexwb
 * @Date: 2026-04-15 23:20
 */

import requestBridge from '/@/utils/requestBridge'
// [迁移调整] 本地派生逻辑统一走 src/bridge 桥接层（Tauri invoke）
import { bridge } from '/@/bridge'

/**
 * 标准分类（所有标准）
 * IPC: standards 表
 * HTTP: /front/standards/standardsAll
 */
export async function standardsAll() {
  const result = await requestBridge.getAll({
    ipc: {
      table: 'standards',
      filters: { status: 1 },
    },
    http: {
      url: '/front/standards/standardsAll',
    },
  })
  return { data: result }
}

/**
 * 形状分类（所有形状）
 * IPC: shapes 表
 * HTTP: /front/standards/shapesAll
 */
export async function shapesAll() {
  const result = await requestBridge.getAll({
    ipc: {
      table: 'shapes',
      filters: { status: 1 },
    },
    http: {
      url: '/front/standards/shapesAll',
    },
  })
  return { data: result }
}

/**
 * 产品分类（所有分类，树形）
 * IPC: categories/products/products_filter 三表本地派生（与服务端 CategoriesQueryBuilder.getTreeLive 语义一致）
 * HTTP: /front/standards/categoriesAll
 */

// IPC 通用解析：本地 db 查询可能返回 { data: [...] } 或裸数组
const ipcRows = (res: any): any[] =>
  Array.isArray(res) ? res : Array.isArray(res?.data) ? res.data : []

/**
 * IPC 直连时本地派生分类树：
 * 1. 启用且未软删的产品为准，取其全部正式关联（products_filter product_type=0）
 * 2. 分类节点 standards = 该分类下产品的 standard_id（去重）
 * 3. 分类节点 shapes = 该分类下产品所关联的全部形状（产品跨分类闭包广播，与后端一致）
 * 4. 按 parent_id 构建层级树
 */
const buildCategoriesTreeLocal = async (): Promise<any[] | null> => {
  // [迁移调整] 统一走 src/bridge 桥接层（本地库不可用时由调用方降级 HTTP）
  const api = bridge
  if (!api?.db?.findAll) return null
  try {
    const [catRes, prodRes, fRes] = await Promise.all([
      api.db.findAll('categories', { status: 1 }),
      api.db.findAll('products', { status: 1 }),
      api.db.findAll('products_filter', { product_type: 0 }),
    ])
    const categories = ipcRows(catRes).filter((c: any) => !c.deleted_at)
    const products = ipcRows(prodRes)
    const filterRows = ipcRows(fRes)

    const enabledProducts = products.filter((p: any) => Number(p.status) === 1 && !p.deleted_at)
    const standardByProduct: Record<number, number | null> = {}
    enabledProducts.forEach((p: any) => {
      standardByProduct[Number(p.id)] = p.standard_id != null || p.standardId != null ? Number(p.standard_id ?? p.standardId) : null
    })
    const enabledProductIds = new Set(enabledProducts.map((p: any) => Number(p.id)))
    const fEnabled = filterRows.filter((r: any) => enabledProductIds.has(Number(r.product_id)))

    // 分类 -> 产品/标准 派生
    const productsByCategory: Record<number, number[]> = {}
    const standardsByCategory: Record<number, number[]> = {}
    fEnabled.forEach((row: any) => {
      if (row.category_id == null) return
      const catId = Number(row.category_id)
      const pid = Number(row.product_id)
      if (!productsByCategory[catId]) productsByCategory[catId] = []
      if (!productsByCategory[catId].includes(pid)) productsByCategory[catId].push(pid)
      const std = standardByProduct[pid]
      if (std != null) {
        if (!standardsByCategory[catId]) standardsByCategory[catId] = []
        if (!standardsByCategory[catId].includes(std)) standardsByCategory[catId].push(std)
      }
    })

    // 形状派生：产品所属分类的闭包广播（与后端 loadCategoryRelationSnapshot 一致）
    const shapesByCategory: Record<number, number[]> = {}
    fEnabled.forEach((row: any) => {
      if (row.shape_id == null) return
      const shapeId = Number(row.shape_id)
      const pid = Number(row.product_id)
      Object.keys(productsByCategory).forEach((key) => {
        const catId = Number(key)
        if (productsByCategory[catId].includes(pid)) {
          if (!shapesByCategory[catId]) shapesByCategory[catId] = []
          if (!shapesByCategory[catId].includes(shapeId)) shapesByCategory[catId].push(shapeId)
        }
      })
    })

    const withDerived = categories.map((c: any) => ({
      ...c,
      standards: standardsByCategory[Number(c.id)] || [],
      shapes: shapesByCategory[Number(c.id)] || [],
    }))

    // 构建层级树（与服务端 buildCategoryTree 同构）
    const buildTree = (all: any[], parentId: number | null): any[] =>
      all
        .filter((item: any) => {
          const pid = item.parent_id ?? item.parentId
          if (parentId === null) return pid == null || Number(pid) === 0
          return Number(pid) === parentId
        })
        .map((item: any) => ({ ...item, children: buildTree(all, Number(item.id)) }))
    return buildTree(withDerived, null)
  } catch (error) {
    console.warn('[categoriesAll IPC 本地派生失败，fallback to HTTP]', error)
    return null
  }
}

export async function categoriesAll() {
  // IPC 直连：本地三表派生树（HTTP 与 IPC 两路语义一致）
  if (requestBridge.isElectron()) {
    const tree = await buildCategoriesTreeLocal()
    if (tree) return { data: tree }
  }
  const result = await requestBridge.getAll({
    http: {
      url: '/front/standards/categoriesAll',
    },
  })
  return { data: result }
}

/**
 * 文档原件下载
 * IPC: fs.readFile（本地缓存）
 * HTTP: /front/standards/docsDownload
 */
export async function docsDownload(data: { id?: number | number[] }) {
  // 优先检查本地缓存
  const cacheKey = `docs/${Array.isArray(data.id) ? data.id.join('-') : data.id}`
  const cached = await requestBridge.readFile({
    ipc: { filePath: cacheKey },
  })
  
  if (cached) {
    return { data: cached }
  }
  
  // Fallback HTTP
  const result = await requestBridge.create({
    http: {
      url: '/front/standards/docsDownload',
      data,
    },
  })
  
  // 缓存结果（仅 IPC 环境）
  if (result && requestBridge.isElectron()) {
    await requestBridge.writeFile({
      ipc: { filePath: cacheKey, data: JSON.stringify(result) },
    })
  }
  
  return { data: result }
}

/**
 * 文档详细
 * IPC: docs 表
 * HTTP: /front/standards/docsDetail
 */
export async function docsDetail(data: { id?: number | number[] }) {
  const id = Array.isArray(data.id) ? data.id[0] : data.id
  const result = await requestBridge.getOne({
    ipc: {
      table: 'docs',
      id,
    },
    http: {
      url: '/front/standards/docsDetail',
      data,
    },
  })
  return { data: result }
}

/**
 * 标准详细
 * IPC: products 表
 * HTTP: /front/standards/productsDetail
 */
export async function productsDetail(data: { id?: number | number[] }) {
  const id = Array.isArray(data.id) ? data.id[0] : data.id
  const result = await requestBridge.getOne({
    ipc: {
      table: 'products',
      id,
    },
    http: {
      url: '/front/standards/productsDetail',
      data,
    },
  })
  return { data: result }
}

/**
 * 标准列表（分页）
 * 
 * @param data 产品临时数据对象
 * @param data.filter.keywords 模糊搜索
 * @param data.filter.id 编号搜索
 * @param data.filter.standard_id 标准分类编号
 * @param data.filter.shape_id 形状分类编号
 * @param data.filter.category_id 产品分类编号
 * @param data.filter.code 产品编码（可选）
 * @param data.filter.status 状态：0禁用，1启用（可选）
 * @param data.sort 排序方式（可选）
 * @param data.page 页码（可选）
 * @param data.pageSize 每页数量（可选）
 * @returns 返回请求的Promise对象，包含列表数据
 */

export async function productsList(data: {
  filter?: {
    keywords?: string
    id?: number | number[] | { not?: number[]; eq?: number[] }
    standard_id?: number | number[]
    shape_id?: number | number[]
    category_id?: number | number[]
    code?: string
    status?: number | string
  }
  sort?: string
  page?: number
  pageSize?: number
}) {
  // 解析排序
  const order = data.sort
    ? [{ column: data.sort.replace(/^[+-]/, ''), order: data.sort.startsWith('-') ? 'DESC' : 'ASC' }]
    : undefined
  
  // 计算偏移量
  const limit = data.pageSize || 20
  const offset = ((data.page || 1) - 1) * limit
  
  const result = await requestBridge.getList({
    ipc: {
      table: 'products',
      filters: data.filter || {},
      order,
      limit,
      offset,
    },
    http: {
      url: '/front/standards/productsList',
      data,
    },
  })
  
  return { data: result }
}

/**
 * 筛选联动可用形状（由 products_filter 实时派生）
 * 替代旧版 standards/shapes.extension.shapes 关系快照：
 * 在已选分类/标准/形状组合下，返回产品仍覆盖的形状ID集合（含已选形状）
 * 
 * @param data.filter.category_id 已选产品分类编号
 * @param data.filter.standard_id 已选标准分类编号
 * @param data.filter.shape_ids 已选形状编号数组
 * @returns { data: { shape_ids: number[] } }
 */
export async function productsFilterOptions(data: {
  filter: {
    category_id?: number | number[]
    standard_id?: number
    shape_ids?: number[]
  }
}) {
  // 聚合计算仅在 HTTP 后端可用；IPC（Electron 本地直连）场景由调用方自行聚合兜底
  const result = await requestBridge.create({
    http: {
      url: '/front/standards/productsFilterOptions',
      data,
    },
  })
  return { data: result }
}

/**
 * 积分购买产品标准（每标准默认 5 积分，可被 products.extension.price_credit 覆盖）
 * 扣费走服务端事务（钱包行锁 + 流水 + customer_users.credit 同步），离线本地库无积分钱包，故仅 HTTP 通道
 * 幂等：订阅有效期内或已购项不重复扣费（返回 charged=false 与 reason）
 * HTTP: /front/standards/productsPurchase
 * @param data.id 产品标准编号
 * @returns { product_id, charged, reason, access, price_credit, use_credits?, balance, version?, purchased_version? }
 *   400030005015 缺少标准编号 / 401030005019 登录态失效 / 400400010002 积分余额不足
 */
export async function productsPurchase(data: { id: number }) {
  const result = await requestBridge.create({
    http: {
      url: '/front/standards/productsPurchase',
      data,
    },
  })
  return { data: result }
}

/**
 * 我的已购产品标准列表（含购买时版本快照 pdf_path / version，与当前最新版本对比）
 * HTTP: /front/standards/purchasedList
 */
export async function purchasedList(data: { page?: number; pageSize?: number } = {}) {
  const result = await requestBridge.create({
    http: {
      url: '/front/standards/purchasedList',
      data,
    },
  })
  return { data: result }
}

/**
 * 产品标准 PDF 下载地址（三态：free_tier=1 免费样例 / 订阅→最新版本 / 已购未订阅→购买时快照版本）
 * HTTP: /front/standards/downloadPdfUrl
 * 403030005016 未订阅且未购买或免费样例不可下载 / 400200010002 该产品暂无 PDF
 */
export async function productsDownloadPdfUrl(data: { id: number }) {
  const result = await requestBridge.create({
    http: {
      url: '/front/standards/downloadPdfUrl',
      data,
    },
  })
  return { data: result }
}

/**
 * 订阅用户标准年度更新列表（订阅专属权益：订阅全年可获得全部标准及当年标准更新）
 * 积分单独购买的标准不进入更新通道（已购用户始终使用购买时快照版本）
 * HTTP: /front/standards/updatesList
 * 未订阅：返回 subscribed=false + 空列表 + guide（前端据此渲染订阅引导，不视为错误）
 * @param data.year 归属年份（默认当年）
 * @param data.type 标准体系（products / materials / capabilities / exteriors，或 1~4）
 * @returns { subscribed, list, total, page, pageSize, year, years, update_window?, guide? }
 */
export async function updatesList(data: { year?: number; type?: string | number; page?: number; pageSize?: number } = {}) {
  const result = await requestBridge.create({
    http: {
      url: '/front/standards/updatesList',
      data,
    },
  })
  return { data: result }
}

// 导出 requestBridge 以便其他模块使用
export { requestBridge }