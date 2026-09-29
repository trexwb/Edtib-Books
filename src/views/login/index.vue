<!--
 * @Author: trexwb
 * @Date: 2023-11-14 11:28:18
 * @LastEditors: ${git_name}
 * @LastEditTime: 2026-09-10 16:44:18
 * @FilePath: /fastenerTradeWorkbench/Users/wbtrex/website/localServer/node/edtib/client/books/web/src/views/login/index.vue
 * @Description: 
 * 一花一世界，一叶一如来
 * Copyright (c) 2024 by 杭州大美, All Rights Reserved. 
-->
<template>
  <div class="login-container">
    <div class="login-form">
      <el-form ref="formRef" label-position="left" :model="form" :rules="dynamicRules" @submit.prevent>
        <!-- 密钥登录为不起眼的备用入口，激活后隐藏常规 tabs -->
        <el-tabs v-if="state.view !== 'secret'" v-model="activeTab">
          <el-tab-pane label="账号密码" name="account">
            <el-form-item prop="username">
              <el-input v-model.trim="form.username" v-focus clearable :placeholder="translate('请输入用户名')" type="text">
                <template #prefix>
                  <vab-icon icon="user-line" />
                </template>
              </el-input>
            </el-form-item>
            <el-form-item prop="password">
              <el-input
                :key="passwordType"
                ref="passwordRef"
                v-model.trim="form.password"
                clearable
                :placeholder="translate('请输入密码')"
                :type="passwordType"
                @keyup.enter="handleLogin"
              >
                <template #prefix>
                  <vab-icon icon="lock-line" />
                </template>
              </el-input>
            </el-form-item>
          </el-tab-pane>
          <el-tab-pane label="短信登录" name="sms">
            <el-form-item prop="mobile">
              <el-input v-model.trim="form.mobile" clearable maxlength="11" :placeholder="translate('请输入手机号')" type="text">
                <template #prefix>
                  <vab-icon icon="smartphone-line" />
                </template>
              </el-input>
            </el-form-item>
            <el-form-item prop="code">
              <el-input
                v-model.trim="form.code"
                clearable
                maxlength="6"
                :placeholder="translate('请输入短信验证码')"
                type="text"
                @keyup.enter="handleSmsLogin"
              >
                <template #prefix>
                  <vab-icon icon="shield-check-line" />
                </template>
                <template #append>
                  <el-button
                    :disabled="loginSms.sending || loginSms.countdown > 0"
                    :loading="loginSms.sending"
                    @click.prevent="handleSendLoginCode"
                  >
                    {{ loginSms.countdown > 0 ? `${loginSms.countdown}s` : translate('获取验证码') }}
                  </el-button>
                </template>
              </el-input>
            </el-form-item>
          </el-tab-pane>
          <el-tab-pane label="注册" name="register">
            <el-form-item prop="mobile">
              <el-input v-model.trim="form.mobile" clearable maxlength="11" :placeholder="translate('请输入手机号')" type="text">
                <template #prefix>
                  <vab-icon icon="smartphone-line" />
                </template>
              </el-input>
            </el-form-item>
            <el-form-item prop="code">
              <el-input
                v-model.trim="form.code"
                clearable
                maxlength="6"
                :placeholder="translate('请输入短信验证码')"
                type="text"
                @keyup.enter="handleRegister"
              >
                <template #prefix>
                  <vab-icon icon="shield-check-line" />
                </template>
                <template #append>
                  <el-button
                    :disabled="registerSms.sending || registerSms.countdown > 0"
                    :loading="registerSms.sending"
                    @click.prevent="handleSendCode"
                  >
                    {{ registerSms.countdown > 0 ? `${registerSms.countdown}s` : translate('获取验证码') }}
                  </el-button>
                </template>
              </el-input>
            </el-form-item>
            <el-form-item prop="registerPassword">
              <el-input
                :key="registerPasswordType"
                ref="registerPasswordRef"
                v-model.trim="form.registerPassword"
                clearable
                :placeholder="translate('请设置密码（不少于6位）')"
                :type="registerPasswordType"
                @keyup.enter="handleRegister"
              >
                <template #prefix>
                  <vab-icon icon="lock-line" />
                </template>
              </el-input>
            </el-form-item>
          </el-tab-pane>
        </el-tabs>
        <!-- 密钥登录（备用入口）：uuid + 密钥 -->
        <template v-else>
          <el-form-item prop="uuid">
            <el-input v-model.trim="form.uuid" v-focus clearable :placeholder="translate('请输入UUID')" type="text">
              <template #prefix>
                <vab-icon icon="pass-expired-fill" />
              </template>
            </el-input>
          </el-form-item>
          <el-form-item prop="secret">
            <el-input
              :key="secretType"
              ref="secretRef"
              v-model.trim="form.secret"
              clearable
              :placeholder="translate('请输入密钥')"
              :type="secretType"
              @keyup.enter="handleLogin"
            >
              <template #prefix>
                <vab-icon icon="key-fill" />
              </template>
            </el-input>
          </el-form-item>
        </template>
        <el-button
          class="login-btn"
          :loading="loading"
          :native-type="viewConfig.nativeType"
          type="primary"
          @click.prevent="viewConfig.submit()"
        >
          {{ translate(viewConfig.label) }}
        </el-button>
        <!-- 密钥登录入口：弱化为不起眼的小链接 -->
        <div class="secret-entry">
          <el-button class="secret-link" link size="small" @click="toggleSecretMode">
            {{ translate(state.view === 'secret' ? '返回常规登录' : '密钥登录') }}
          </el-button>
        </div>
        <vab-slider-verify :show="state.verifyShow" @close="onClose" @fail="onFail" @success="onSuccess" />
      </el-form>
    </div>
  </div>
</template>

<script lang="ts" setup>
import VabSliderVerify from 'vue3-puzzle-vcode'
import { register, signSms } from '/@/api/authorize'
import { tokenName } from '/@/config'
import { themeConfig } from '/@/config/theme.config'
import { translate } from '/@/i18n'
import { useSettingsStore } from '/@/store/modules/settings'
import { useUserStore } from '/@/store/modules/user'
import { getStorage, setStorage } from '/@/utils/storage'
import { isPassword, isPhone } from '/@/utils/validate'
import { SMS_SCENE, useSmsCode } from './useSmsCode'

import { log } from '/@/utils'
log.info('奥德彪', '我并非无路可走 我还有死路一条! ')
log.error('奥德彪', '钱没了可以再赚，良心没了便可以赚的更多。 ')
log.warning('奥德彪', '前方的路看似很危险,实际一点也不安全。 ')
log.success('奥德彪', '出来的时候穷 生活总是让我穷 所以现在还是穷。')
log.picture(
  'https://nimg.ws.126.net/?url=http%3A%2F%2Fdingyue.ws.126.net%2F2024%2F0514%2Fd0ea93ebj00sdgx56001xd200u000gtg00hz00a2.jpg&thumbnail=660x2147483647&quality=80&type=jpg'
)

defineOptions({
  name: 'Login',
})

const $baseMessage = inject<any>('$baseMessage')

type LoginView = 'account' | 'sms' | 'register' | 'secret'

const state = reactive({
  // 单一视图状态：常规三 tab（account / sms / register）+ 密钥登录入口（secret）
  view: 'account' as LoginView,
  verifyShow: false,
  verifySuccess: false,
})

// 短信验证码：登录（scene=1，手机号须已注册）与注册（scene=2）各持一份独立实例
const loginSms = useSmsCode(SMS_SCENE.LOGIN, { message: $baseMessage })
const registerSms = useSmsCode(SMS_SCENE.REGISTER, { message: $baseMessage })

// el-tabs 的 v-model 为 string | number，此处收窄回 LoginView，避免写入非法视图值
const activeTab = computed({
  get: () => state.view as string,
  set: (name: string | number) => {
    state.view = name as LoginView
  },
})

const onShow = () => {
  state.verifyShow = true
}

const onClose = () => {
  state.verifyShow = false
}

const onSuccess = () => {
  state.verifySuccess = true
  onClose()
  handleLogin()
}

const onFail = () => {
  state.verifySuccess = false
  $baseMessage('验证失败，请重试', 'error', 'hey')
}

const route = useRoute()
const router = useRouter()
const userStore = useUserStore()
const settingsStore = useSettingsStore()
const login = (form: any) => userStore.login(form)
const { title } = storeToRefs(settingsStore)
const loading = ref<boolean>(false)
const passwordType = ref<string>('password')
const secretType = ref<string>('password')
const registerPasswordType = ref<string>('password')

const redirect = ref<any>(undefined)
const { handleUnLock } = settingsStore
// let timer: any
// const codeUrl = ref<string>('https://www.oschina.net/action/user/captcha')
// const previewText = ref<string>('')
const formRef = ref<any>(null)
const passwordRef = ref<any>(null)
const secretRef = ref<any>(null)
const registerPasswordRef = ref<any>(null)
const loginFormData = getStorage('loginFormData') || {}
const form = ref<any>(
  Object.assign(
    {
      username: loginFormData['username'] || (process.env.NODE_ENV === 'development' ? 'wangbin@edtib.com' : ''),
      password: process.env.NODE_ENV === 'development' ? 'WxzLyR3PPRFvPm8f' : '',
      uuid: '',
      secret: '',
      verificationCode: '',
      mobile: '',
      code: '',
      registerPassword: '',
    },
    loginFormData
  )
)

// 更换手机号即作废旧验证码：清空 token 与倒计时，避免用旧 token 登录/注册新号
watch(
  () => form.value.mobile,
  () => {
    loginSms.reset()
    registerSms.reset()
  }
)

const validateUsername = (rule: any, value: any, callback: any) => {
  if ('' === value) callback(new Error(translate('用户名不能为空')))
  else callback()
}
const validatePassword = (rule: any, value: any, callback: any) => {
  if (!isPassword(value)) callback(new Error(translate('密码不能少于6位')))
  else callback()
}

const accountRules = reactive<any>({
  username: [
    {
      required: true,
      trigger: 'blur',
      validator: validateUsername,
    },
  ],
  password: [
    {
      required: true,
      trigger: 'blur',
      validator: validatePassword,
    },
  ],
})
const secretRules = reactive<any>({
  uuid: [
    {
      required: true,
      trigger: 'blur',
    },
  ],
  secret: [
    {
      required: true,
      trigger: 'blur',
    },
  ],
})

const validateMobile = (rule: any, value: any, callback: any) => {
  if ('' === value) callback(new Error(translate('手机号不能为空')))
  else if (!isPhone(value)) callback(new Error(translate('手机号格式不正确')))
  else callback()
}
const validateSmsCode = (rule: any, value: any, callback: any) => {
  if ('' === value) callback(new Error(translate('验证码不能为空')))
  else callback()
}
/**
 * 手机号 + 短信验证码：登录与注册共用同一份校验规则，避免逐字段重复
 */
const mobileCodeRules: any = {
  mobile: [
    {
      required: true,
      trigger: 'blur',
      validator: validateMobile,
    },
  ],
  code: [
    {
      required: true,
      trigger: 'blur',
      validator: validateSmsCode,
    },
  ],
}

const smsLoginRules = mobileCodeRules

const registerRules: any = {
  ...mobileCodeRules,
  registerPassword: [
    {
      required: true,
      trigger: 'blur',
      validator: validatePassword,
    },
  ],
}

/**
 * 密钥登录（备用入口）：隐藏常规 tabs，仅保留 uuid + 密钥；退出固定回账号密码
 */
const toggleSecretMode = () => {
  state.view = state.view === 'secret' ? 'account' : 'secret'
}

const handleRoute = () => {
  return redirect.value === '/404' || redirect.value === '/403' ? '/' : redirect.value
}

const handleLogin = async () => {
  if (loading.value) return
  if (!state.verifySuccess) {
    onShow()
    return
  }
  if (formRef.value && state.verifySuccess)
    formRef.value.validate(async (valid: any) => {
      if (valid)
        try {
          // 滑块验证只授权本次尝试，失败重试需重新验证
          state.verifySuccess = false
          setStorage('color', themeConfig.color)
          setStorage('loginFormData', { username: form.value.username })
          loading.value = true
          if (state.view === 'secret') {
            await login({
              uuid: form.value.uuid,
              secret: form.value.secret,
            })
              .then(async () => {
                $baseMessage('验证成功', 'success', 'hey')
                await router.push(handleRoute())
                handleUnLock()
              })
              .catch((error: any) => {
                // $baseMessage(error.toString(), 'error', 'hey')
                form.value.secret = ''
                loading.value = false
              })
          } else {
            await login({
              username: form.value.username,
              password: form.value.password,
            })
              .then(async () => {
                $baseMessage('验证成功', 'success', 'hey')
                await router.push(handleRoute())
                handleUnLock()
              })
              .catch((error: any) => {
                // $baseMessage(error.toString(), 'error', 'hey')
                form.value.password = process.env.NODE_ENV === 'development' ? 'WxzLyR3PPRFvPm8f' : ''
                loading.value = false
              })
          }
        } finally {
          loading.value = false
        }
    })
}
/**
 * 短信登录：发送短信验证码（登录场景，手机号须已注册）
 * 手机号校验、下发、token 缓存、倒计时与兜底统一由 useSmsCode 处理
 */
const handleSendLoginCode = () => loginSms.send(form.value.mobile)

/**
 * 短信登录：手机号 + 短信验证码
 * 短信验证码本身即人机校验，不走滑块验证；成功返回与密码登录同构的登录态
 */
const handleSmsLogin = async () => {
  if (loading.value) return
  if (!loginSms.token) {
    $baseMessage(translate('请先获取短信验证码'), 'warning', 'hey')
    return
  }
  if (!formRef.value) return
  formRef.value.validate(async (valid: any) => {
    if (!valid) return
    loading.value = true
    try {
      const { data } = await signSms({
        mobile: form.value.mobile,
        token: loginSms.token,
        code: form.value.code,
      })
      if (!data?.auth_token) {
        $baseMessage(translate('登录接口异常，未正确返回登录态'), 'error', 'hey')
        return
      }
      loginSms.reset()
      setStorage('color', themeConfig.color)
      userStore.afterLogin(data.auth_token, tokenName)
      await router.push(handleRoute())
      handleUnLock()
    } catch (error: any) {
      // 业务错误已由请求拦截器统一提示；仅验证码已作废时才清 token，其余保留供重试
      form.value.code = ''
      loginSms.handleError(error)
    } finally {
      loading.value = false
    }
  })
}

/**
 * 注册：发送短信验证码
 */
const handleSendCode = () => registerSms.send(form.value.mobile)

/**
 * 注册：手机号 + 短信验证码 + 设置密码
 * 注册成功后端直接返回与登录同构的登录态，写入 token 后进入首页
 */
const handleRegister = async () => {
  if (loading.value) return
  if (!registerSms.token) {
    $baseMessage(translate('请先获取短信验证码'), 'warning', 'hey')
    return
  }
  if (!formRef.value) return
  formRef.value.validate(async (valid: any) => {
    if (!valid) return
    loading.value = true
    try {
      const { data } = await register({
        mobile: form.value.mobile,
        token: registerSms.token,
        code: form.value.code,
        password: form.value.registerPassword,
      })
      if (!data?.auth_token) {
        $baseMessage(translate('注册接口异常，未正确返回登录态'), 'error', 'hey')
        return
      }
      registerSms.reset()
      setStorage('color', themeConfig.color)
      userStore.afterLogin(data.auth_token, tokenName)
      $baseMessage('注册成功', 'success', 'hey')
      await router.push(handleRoute())
      handleUnLock()
    } catch (error: any) {
      // 业务错误已由请求拦截器统一提示；仅验证码已作废时才清 token，其余保留供重试
      form.value.code = ''
      registerSms.handleError(error)
    } finally {
      loading.value = false
    }
  })
}

// const changeCode = () => {
//   codeUrl.value = `https://www.oschina.net/action/user/captcha?timestamp=${new Date().getTime()}`
// }

// onBeforeMount(() => {
//   form.value.username = 'admin'
//   form.value.password = '123456'
//   // 为了演示效果，会在官网演示页自动登录到首页，正式开发可删除
//   if (location.hostname === 'gateway-dev.edtib.com' || location.hostname === 'chu1204505056.gitee.io') {
//     previewText.value = '（演示地址验证码可不填）'
//     timer = setTimeout(() => {
//       handleLogin()
//     }, 5000)
//   }
// })

watchEffect(() => {
  redirect.value = (route.query && route.query.redirect) || '/'
})

/**
 * 视图 → 校验规则 / 提交动作 / 按钮文案 的唯一映射表
 * 新增视图只需加一行，无需在模板与 rules 之间同步改多处分支
 * 依赖下方的 handler，故置于文件末尾
 */
const VIEW_ACTIONS: Record<
  LoginView,
  {
    rules: any
    submit: () => void
    label: string
    nativeType: 'submit' | 'button'
  }
> = {
  account: { rules: accountRules, submit: handleLogin, label: '登录', nativeType: 'submit' },
  sms: { rules: smsLoginRules, submit: handleSmsLogin, label: '登录', nativeType: 'button' },
  register: { rules: registerRules, submit: handleRegister, label: '注册并登录', nativeType: 'button' },
  secret: { rules: secretRules, submit: handleLogin, label: '登录', nativeType: 'submit' },
}

const viewConfig = computed(() => VIEW_ACTIONS[state.view])
const dynamicRules = computed(() => viewConfig.value.rules)
</script>

<style lang="scss" scoped>
.login-container {
  position: relative;
  height: 100vh;
  background: linear-gradient(to top, var(--el-color-primary), var(--el-color-primary-light-3));

  .login-right-tools {
    position: fixed;
    top: var(--el-margin);
    right: var(--el-margin);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: calc(var(--el-padding) / 2) var(--el-padding) calc(var(--el-padding) / 2) var(--el-padding);
    background: var(--el-color-white);
    border: 1px solid var(--el-border-color);
    border-radius: var(--el-border-radius-base);
  }

  @media (max-width: 768px) {
    .login-right-tools {
      top: 5vw !important;
      right: 5vw !important;
    }

    .login-form {
      width: 90vw !important;
      margin: auto !important;

      .left-img {
        display: none !important;
      }

      :deep() {
        .el-form--default {
          width: 100% !important;
          margin-right: auto !important;
          margin-left: auto !important;
        }
      }
    }
  }

  .login-form {
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    left: 0;
    width: 500px;
    height: 500px;
    padding: 4.5vh;
    margin: auto;
    overflow: hidden;
    background: var(--el-color-white);
    background-size: 100% 100%;
    border: 1px solid var(--el-border-color);
    border-radius: 15px;

    .left-img {
      float: left;
      width: 50%;
    }

    :deep() {
      .el-form--default {
        width: 90%;
        margin: 0 auto;
      }

      .title {
        font-size: 54px;
        font-weight: 500;
        color: var(--el-color-black);
      }

      .title-tips {
        margin-top: 29px;
        font-size: 26px;
        font-weight: 400;
        color: var(--el-color-black);
      }

      .login-btn {
        width: 100%;
        height: 50px;
      }

      .el-form-item {
        margin: 20px 0;

        &__error {
          position: absolute;
          font-size: var(--el-font-size-small);
          line-height: 18px;
          color: var(--el-color-error);
        }

        .el-input {
          width: 100%;
          height: 48px;
          line-height: 48px;
        }
      }

      .code {
        position: absolute;
        top: 4px;
        right: 4px;
        cursor: pointer;
        border-radius: var(--el-border-radius-base);
      }
    }

    // 组件自身元素，无需 :deep() 穿透
    .secret-entry {
      margin-top: -2px;
      text-align: center;

      .secret-link {
        height: auto;
        padding: 0;
        font-size: 12px;
        color: var(--el-text-color-secondary);
        transition: color 0.2s ease;

        &:hover {
          color: var(--el-color-primary);
        }
      }
    }
  }
}
</style>
