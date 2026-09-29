<!--
 * @Author: ${git_name}
 * @Date: 2025-04-09 08:45:50
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-07-07 09:19:13
 * @FilePath: /books/web/src/views/standards/products/vabAutoComponents/ProductsStlViewer.vue
 * @Description: 
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved. 
-->
<template>
  <div v-show="!state.loading" class="stl-container">
    <div ref="viewportRef" class="viewport"></div>
  </div>
</template>

<script setup>
import * as THREE from 'three'
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js'
import { STLLoader } from 'three/examples/jsm/loaders/STLLoader.js'
import { evaluateConditionDefault } from '/@/utils/evaluateCondition'

const props = defineProps(['productRow', 'queryForm'])
// 类型守卫：必须 string 才是合法 URL，对象/数组/空串都丢弃，避免 STLLoader 拿到非 STL 数据
// 触发 RangeError: Invalid typed array length（头部 80 字节被读成巨大 faces 数）
const rawUrl = props.productRow?.extension?.models ?? props.productRow?.models?.[0]?.url ?? ''
const modelUrl = ref(typeof rawUrl === 'string' && /^https?:\/\//.test(rawUrl) ? rawUrl : '')

const state = reactive({
  loading: false,
})

const viewportRef = ref(null)
let scene, camera, renderer, controls, mesh

const initThree = () => {
  // 创建场景
  scene = new THREE.Scene()

  // 创建相机
  const aspect = window.innerWidth / window.innerHeight
  camera = new THREE.PerspectiveCamera(45, aspect, 0.1, 1000)
  camera.position.set(5, 5, 5)

  // 创建渲染器
  renderer = new THREE.WebGLRenderer({ antialias: true })
  renderer.setSize(window.innerWidth, window.innerHeight)
  renderer.setClearColor(0xeaf0f0) // 设置背景颜色
  viewportRef.value.appendChild(renderer.domElement)

  // 添加光源
  const ambientLight = new THREE.AmbientLight(0xffffff, 0.5)
  scene.add(ambientLight)
  const directionalLight = new THREE.DirectionalLight(0xffffff, 1)
  directionalLight.position.set(3, 3, 3)
  scene.add(directionalLight)

  // 初始化控制器
  controls = new OrbitControls(camera, renderer.domElement)
  controls.enableDamping = true
  controls.dampingFactor = 0.05
  controls.enablePan = false
  controls.minPolarAngle = 0
  controls.maxPolarAngle = Math.PI * 2

  // 加载模型
  loadModel()

  animate()
}

const loadModel = () => {
  state.loading = true
  const loader = new STLLoader()
  const material = new THREE.MeshStandardMaterial({
    color: 0xf3f3f3,
    metalness: 0.2,
    roughness: 0.3,
  })

  if (!modelUrl.value) {
    // URL 非法（类型守卫过滤掉了对象/数组/空串），直接隐藏 viewer
    state.loading = false
    return
  }

  loader.load(modelUrl.value, (geometry) => {
    if (mesh) {
      scene.remove(mesh)
      mesh.geometry.dispose()
      mesh.material.dispose()
    }

    geometry.center() // 自动将几何体中心移到原点
    mesh = new THREE.Mesh(geometry, material)

    // 预旋转模型（根据需要调整）
    mesh.rotation.x = Math.PI / 2

    scene.add(mesh)

    // 计算边界盒并调整相机位置以适应模型大小
    geometry.computeBoundingBox()
    const size = geometry.boundingBox.getSize(new THREE.Vector3())
    const maxDim = Math.max(size.x, size.y, size.z)
    const cameraDistance = maxDim * 1.5
    camera.position.set(cameraDistance, cameraDistance, cameraDistance)
    camera.lookAt(0, 0, 0)
    controls.target.set(0, 0, 0)
    // controls.update()
    state.loading = false
  })
}

let animationFrameId = null
const animate = () => {
  animationFrameId = requestAnimationFrame(animate)
  controls && controls.update() // 更新控制器
  renderer && renderer.render(scene, camera)
}
// 已有渲染循环则不重复启动（多次 showImage 只保留一个循环）
const startAnimate = () => {
  if (animationFrameId === null) animate()
}
// 停止渲染循环并释放资源
const stopAnimate = () => {
  if (animationFrameId !== null) {
    cancelAnimationFrame(animationFrameId)
    animationFrameId = null
  }
}
// 处理窗口大小变化
const handleResize = () => {
  const aspect = window.innerWidth / window.innerHeight
  camera.aspect = aspect
  camera.updateProjectionMatrix()
  renderer.setSize(window.innerWidth, window.innerHeight)
}

// 删除 STL 模型
const removeModel = () => {
  if (mesh) {
    scene.remove(mesh)
  }
}

const diameterChange = () => {
  const { productRow, queryForm } = props
  if (productRow?.models.length > 1 && productRow?.drawingLimit?.[queryForm?.mon]?.['IMG']) {
    modelUrl.value = evaluateConditionDefault(productRow.drawingLimit?.[queryForm?.mon]['IMG'], productRow.parameters, queryForm)
      ? productRow.models[0]?.url || false
      : productRow.models[1]?.url || false
  } else {
    modelUrl.value = productRow?.extension?.models || productRow?.models[0]?.url || false
  }
}
const showImage = () => {
  diameterChange()
  if (modelUrl.value) {
    loadModel()
    startAnimate()
  } else {
    removeModel()
  }
}

// 暴露组件方法给父组件
defineExpose({ showImage })

onMounted(() => {
  initThree()
  window.addEventListener('resize', handleResize)
})

onUnmounted(() => {
  stopAnimate()
  window.removeEventListener('resize', handleResize)
  controls && controls.dispose()
  renderer && renderer.dispose()
  renderer && renderer.forceContextLoss && renderer.forceContextLoss()
  if (mesh) {
    scene.remove(mesh)
    mesh.geometry.dispose()
    mesh.material.dispose()
  }
})
</script>

<style lang="scss">
.stl-container {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;

  .viewport {
    position: relative;
    flex: 1;
    width: 100% !important;
    height: calc(100% - 38px) !important;
  }

  canvas {
    width: 100% !important;
    height: 100% !important;
  }

  button {
    padding: 8px 16px;
    color: white;
    cursor: pointer;
    background: #555;
    border: none;
    border-radius: 4px;
  }
}
</style>
