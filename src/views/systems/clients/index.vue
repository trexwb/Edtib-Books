<template>
  <div class="list-container table-auto-height clients-box">
    <el-tabs v-model="TabsActiveName" class="clients-tabs">
      <template v-for="(item, index) in clientsTabs" :key="'credits' + index">
        <el-tab-pane :name="item.value">
          <template #label>
            <div class="tabs-label">
              <template v-if="index == 0">
                <el-icon>
                  <setting />
                </el-icon>
              </template>
              <template v-if="index == 1">
                <el-icon>
                  <info-filled />
                </el-icon>
              </template>
              <el-text>{{ item.label }}</el-text>
            </div>
          </template>
          <template v-if="TabsActiveName == 'convention'">
            <el-row :gutter="20">
              <el-col :lg="8" :md="8" :sm="8" :xl="8" :xs="8">更新数据（最后一次更新{{ appData.lastTime }}）</el-col>
              <el-col :lg="16" :md="16" :sm="16" :xl="16" :xs="16">
                <el-button :disabled="state.submit" type="default" @click="dataClean">清空全部数据</el-button>
              </el-col>
            </el-row>
            <!-- <el-row :gutter="20">
              <el-col :xl="8" :lg="8" :md="8" :sm="8" :xs="8">清空全部数据</el-col>
              <el-col :xl="16" :lg="16" :md="16" :sm="16" :xs="16">
                <el-button type="default" :disabled="state.submit" @click="dataClean">清空</el-button>
              </el-col>
            </el-row> -->
            <el-row :gutter="20">
              <el-col :lg="8" :md="8" :sm="8" :xl="8" :xs="8">退出账号</el-col>
              <el-col :lg="16" :md="16" :sm="16" :xl="16" :xs="16">
                <el-button type="default" @click="checkLogout">退出</el-button>
              </el-col>
            </el-row>
            <el-row :gutter="20">
              <el-col :lg="8" :md="8" :sm="8" :xl="8" :xs="8">语言</el-col>
              <el-col :lg="16" :md="16" :sm="16" :xl="16" :xs="16">
                <el-button disabled type="default">中文</el-button>
              </el-col>
            </el-row>
            <el-row :gutter="20">
              <el-col :lg="8" :md="8" :sm="8" :xl="8" :xs="8">版本：</el-col>
              <el-col :lg="16" :md="16" :sm="16" :xl="16" :xs="16">
                <clients-check-update />
                <!-- <el-button v-if="appVersion.includes(appPackage.version)" type="text" :disabled="state.submit"
                  @click="checkUpdate">检查更新</el-button> -->
                <!-- <el-button v-else type="text" @click="openUrl(appPackage.packagePath)">
                  下载新版本{{ appPackage.version }}
                </el-button> -->
              </el-col>
            </el-row>
          </template>
          <template v-if="TabsActiveName == 'about'">
            <h2>《紧固助手》提供四大核心功能：</h2>
            <p>1）标准化数据库集成GB/ISO等万条标准参数查询；</p>
            <p>2）智能选型通过材料、载荷等条件自动推荐紧固件，含可靠性计算模块；</p>
            <p>3）装配工艺支持生成扭矩参数及安装指导；</p>
            <p>4）案例库以三维模型展示典型应用场景。</p>
            <p><el-text class="mx-1" type="warning">实现从选型到装配的全流程智能化支持。</el-text></p>
            <el-divider />
            <h2>该平台具备四大特色：</h2>
            <p>1）私域定制，支持企业自有标准库搭建；</p>
            <p>2）‌智能推荐，基于实际工况精准匹配合适的产品；</p>
            <p>3）‌虚拟验证，通过应力仿真降低试验成本；</p>
            <p>4）‌全行业覆盖，覆盖适配航空、化工、新能源、汽车等场景，实现跨领域技术迁移。</p>
            <el-divider />
            <h2>该平台覆盖三大核心场景：</h2>
            <p>1）‌设计阶段，‌自动匹配标准件并生成可靠性报告；</p>
            <p>2）‌生产阶段，‌精准输出扭矩参数减少装配失误；</p>
            <p>3）‌运维阶段，‌通过案例库智能诊断故障。</p>
            <p>特别适用于航空、能源等对安全性要求极高的领域，实现全生命周期效率提升。</p>
          </template>
        </el-tab-pane>
      </template>
    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { Setting, InfoFilled } from '@element-plus/icons-vue'
import { updateLastTime, cachesClear } from '/@/api/systems'
import { configEnum } from '/@/api/common'
import { useUserStore } from '/@/store/modules/user'
import { ElMessageBox, ElMessage } from 'element-plus'
import { toLoginRoute } from '/@/utils/routes'
// [迁移调整] 版本/更新相关 IPC 调用统一走 src/bridge 桥接层（Tauri invoke）
import { bridge } from '/@/bridge'

const templateName = 'Clients'
defineOptions({
  name: templateName,
})

const route = useRoute()
const router = useRouter()
const userStore = useUserStore()
const { avatar, username } = storeToRefs(userStore)
const { logout } = userStore

const appData = ref<any>({})
const appPackage = ref<any>({})
const appVersion = ref('')
const state = reactive<any>({
  submit: false,
  loading: false,
})

const TabsActiveName = ref('convention')
const clientsTabs = reactive([
  {
    label: '常规设置',
    value: 'convention',
  },
  {
    label: '关于我们',
    value: 'about',
  },
])

const checkUpdate = async () => {
  if (state.submit) return
  state.submit = true
  // [迁移调整] IPC 调用统一走 src/bridge 桥接层（Tauri invoke）
  await bridge.checkUpdate()
  state.submit = false
}

const dataClean = async () => {
  ElMessageBox.confirm('清空数据后会重新初始化数据，确定要清空数据吗？', {
    draggable: false,
    type: 'warning',
  })
    .then(async () => {
      if (state.submit) return
      state.submit = true
      await cachesClear()
      ElMessage({
        message: '清空成功',
        type: 'success',
      })
      ElMessageBox.confirm('即将重启应用，确定要重启吗？', {
        draggable: false,
      })
        .then(async () => {
          await bridge.restartApp()
        })
        .catch(() => {
          /* 取消关闭操作 */
        })
    })
    .catch(() => {
      // catch error
    })
    .finally(() => {
      state.submit = false
    })
}

const checkLogout = async () => {
  ElMessageBox.confirm('您确定要放弃登录退出系统吗？', {
    draggable: false,
    type: 'warning',
  })
    .then(async () => {
      await logout()
      await router.push(toLoginRoute(route.fullPath))
    })
    .catch(() => {
      // catch error
    })
}

const openUrl = async (appUrl: string) => {
  const response = await fetch(appUrl)
  if (!response.ok) {
    throw new Error(`Failed to fetch Setup: ${response.statusText}`)
  }
  const blob = await response.blob()
  // 创建一个临时 URL 指向 Blob 数据
  const url = window.URL.createObjectURL(blob)
  // 动态创建一个 HTML <a> 元素，用于触发下载
  const link = document.createElement('a')
  link.href = url
  // 设置下载的文件名
  const fileName = `setup-v${appPackage.value.version}exe`
  link.setAttribute('download', fileName)
  // 将链接添加到 DOM 中，并触发点击事件
  document.body.appendChild(link)
  link.click()
  // 清理：移除链接和释放临时 URL
  document.body.removeChild(link)
  window.URL.revokeObjectURL(url)
}

const getConfig = async () => {
  if (state.loading) return
  state.loading = true
  const { data } = await configEnum()
  state.loading = false
  return data
}
const getUpdateLastTime = async () => {
  if (state.loading) return
  state.loading = true
  const { data } = await updateLastTime()
  state.loading = false
  return data
}

const init = async () => {
  const configRow = await getConfig()
  appPackage.value = configRow.package || {}
  const lastRow = await getUpdateLastTime()
  appData.value.lastTime = lastRow.lastTime || ''
}

onMounted(async () => {
  init()
})
</script>

<style lang="scss">
.clients-box {
  background: rgb(255, 255, 255);

  .tabs-label {
    display: flex;
    align-items: center;
    cursor: pointer;
    user-select: none;

    .el-icon {
      font-size: 20px;
    }
  }

  .el-tabs__item {
    width: 130px;
    padding: 0;
    text-align: center;

    &.is-active {
      background: #eaf0f0;

      .el-text {
        color: #052965;
      }
    }
  }

  .el-row {
    align-items: center;
    padding: 20px 0;
  }

  .name {
    font-size: 16px;
    font-weight: bold;
    color: #052965;
  }

  .operate {
    font-size: 16px;
    color: #052965;
  }
}
</style>
