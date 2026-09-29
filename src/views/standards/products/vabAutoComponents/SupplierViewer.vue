<template>
  <el-card shadow="always" style="width: 100%">
    <template #header>
      <div class="card-header">
        以下是
        <span>“GB/T 5783-2016 全螺纹六角头螺栓”</span>
        的优质供应商
      </div>
    </template>
    <el-row :gutter="20" style="width: 100%">
      <el-col v-for="(item, index) in 5" :key="index" :lg="24" :md="24" :sm="24" :xl="24" :xs="24">
        <el-card shadow="hover" style="width: 100%">
          <el-descriptions style="width: 100%">
            <el-descriptions-item :rowspan="4" :width="160">
              <el-image :src="companyImg" style="width: 140px; height: auto" />
            </el-descriptions-item>
            <el-descriptions-item :span="24">
              <el-text size="large">重庆宇声有色金属有限公司</el-text>
            </el-descriptions-item>
            <el-descriptions-item :span="24">
              <el-button style="color: #666" type="text">
                核心产品: 六角螺母,六角头螺栓,开槽螺钉,十字槽螺钉,牙棒/牙条,平垫,弹簧垫圈,其他自攻/木螺钉,盖形螺母
              </el-button>
            </el-descriptions-item>
            <el-descriptions-item :span="24">
              <el-button style="color: #666" type="text">联系人: 孙竞</el-button>
              <el-button style="color: #666" type="text">手机: 13318312035</el-button>
              <el-button style="color: #666" type="text">电话: 023-62858399</el-button>
            </el-descriptions-item>
            <el-descriptions-item :span="24">
              <el-button style="color: #666" type="text">材料: 硅青铜；</el-button>
              <el-button style="color: #666" type="text">表面处理: 本色；</el-button>
              <el-button style="color: #666" type="text">材质: 4.8 级；</el-button>
              <el-button style="color: #666" type="text">直径: M12；</el-button>
              <el-button style="color: #666" type="text">长度: 16mm-650mm</el-button>
            </el-descriptions-item>
          </el-descriptions>
        </el-card>
      </el-col>
    </el-row>
  </el-card>
</template>

<script setup lang="ts">
import companyImg from '/@/assets/company.jpg'
</script>

<style lang="scss" scoped>
.card-header {
  font-size: 14px;
  span {
    font-weight: bold;
    color: var(--default-color);
  }
}
</style>
