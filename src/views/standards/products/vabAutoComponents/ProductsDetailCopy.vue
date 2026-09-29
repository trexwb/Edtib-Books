<!--
 * @Author: ${git_name}
 * @Date: 2025-04-14 09:20:55
 * @LastEditors: ${git_name}
 * @LastEditTime: 2025-04-29 15:03:00
 * @FilePath: /books/web/src/views/standards/products/vabAutoComponents/ProductsDetailCopy.vue
 * @Description: 
 * 一花一世界，一叶一如来
 * Copyright (c) 2025 by 杭州大美, All Rights Reserved. 
-->
<template>
  <el-drawer
    v-model="state.drawerdFormVisible"
    append-to-body
    :before-close="handleClose"
    :close-on-click-modal="false"
    direction="rtl"
    :loading="config.loading"
    modal-class="products-detail-drawer"
    size="88%"
    :title="config.title"
  >
    <section class="products-detail-drawer-content">
      <el-row :gutter="20">
        <!-- 左侧固定导航 -->
        <el-col :lg="3" :md="3" :sm="3" :xl="3" :xs="3">
          <detail-left-menu
            :side-navigation-tabs-active="sideNavigationTabsActive"
            :side-navigation-tabs-list="sideNavigationTabsList"
            @change-left-menu-tab="changeLeftMenuTab"
          />
        </el-col>
        <!-- 右侧内容 -->
        <el-col :lg="21" :md="21" :sm="21" :xl="21" :xs="21">
          <!-- 基本信息 -->
          <div v-show="sideNavigationTabsActive == 'Basic'">
            <el-card shadow="always" style="width: 100%">
              <h2 class="name-text">ASME/ANSI B18.2.1-3-2012</h2>
              <div class="en-name">High Strength Bolts width Large Hexagon Head Assemblies for Steel</div>
              <el-tabs v-model="imgTabsActive">
                <template v-for="(item, index) in imgTabsList" :key="'img' + index">
                  <el-tab-pane :name="item.name">
                    <template #label>
                      <el-text class="tabs-label">{{ item.label }}</el-text>
                    </template>
                    <el-row>
                      <el-col :lg="24" :md="24" :sm="24" :xl="24" :xs="24">
                        <div class="tabs-content">
                          <!-- 尺寸图 -->
                          <template v-if="imgTabsActive == 'Size'">
                            <size-viewer />
                          </template>
                          <!-- 三维图 -->
                          <template v-if="imgTabsActive == '3D'">
                            <stl-viewer />
                          </template>
                          <!-- CAD图 -->
                          <template v-if="imgTabsActive == 'CAD'">
                            <cad-viewer />
                          </template>
                          <right-content />
                        </div>
                      </el-col>
                    </el-row>
                  </el-tab-pane>
                </template>
              </el-tabs>
              <div class="under-tabs-select-box" style="padding-left: 10px">
                <el-row :gutter="20">
                  <el-col :lg="6" :md="6" :sm="7" :xl="6" :xs="24">
                    <el-form class="custom-el-form" label-width="80px">
                      <el-form-item style="padding-top: 10px; margin-bottom: 10px">
                        <template #label>
                          <div class="under-tabs-img-box">
                            <div>直径</div>
                            <img alt="直径" :src="diameterImg" />
                          </div>
                        </template>
                        <el-select v-model="queryForm.diameter" clearable filterable placeholder="直径" @change="handleDiameterChange">
                          <el-option
                            v-for="item in dropDownDataSet.diameterArr"
                            :key="item.value"
                            :label="item.label"
                            :value="item.value"
                          />
                        </el-select>
                      </el-form-item>
                    </el-form>
                  </el-col>
                  <el-col :lg="6" :md="6" :sm="7" :xl="6" :xs="24">
                    <el-form class="custom-el-form" label-width="80px">
                      <el-form-item style="padding-top: 10px; margin-bottom: 10px">
                        <template #label>
                          <div class="under-tabs-img-box">
                            <div>长度</div>
                            <img alt="长度" :src="lengthImg" />
                          </div>
                        </template>
                        <el-select v-model="queryForm.length" clearable filterable placeholder="长度">
                          <el-option v-for="item in dropDownDataSet.lengthArr" :key="item.value" :label="item.label" :value="item.value" />
                        </el-select>
                      </el-form-item>
                    </el-form>
                  </el-col>
                  <el-col :lg="6" :md="6" :sm="7" :xl="6" :xs="24">
                    <el-form class="custom-el-form" label-width="80px">
                      <el-form-item style="padding-top: 10px; margin-bottom: 10px">
                        <template #label>
                          <div class="under-tabs-img-box">
                            <div>螺距</div>
                            <img alt="螺距" :src="pitchImg" />
                          </div>
                        </template>
                        <el-select v-model="queryForm.pitch" clearable filterable placeholder="螺距" @change="handlePitchChange">
                          <el-option v-for="item in dropDownDataSet.pitchArr" :key="item.value" :label="item.label" :value="item.value" />
                        </el-select>
                      </el-form-item>
                    </el-form>
                  </el-col>
                  <el-col :lg="1" :md="1" :offset="3" :sm="3" :xl="1" :xs="4">
                    <el-tooltip effect="light" :offset="0" placement="top">
                      <template #content>
                        免责申明：本网站所有资源仅供参考学习，
                        <br />
                        禁止未经授权的商业用途，
                        <br />
                        如因依赖从本网站获得的信息并用于商业用途造成的损失，
                        <br />
                        我们对此不负任何责任。
                      </template>
                      <div class="notice-title">
                        <el-icon color="#fff" size="18">
                          <question-filled />
                        </el-icon>
                        <div class="label-title">商业用途</div>
                      </div>
                    </el-tooltip>
                  </el-col>
                </el-row>
              </div>
            </el-card>
            <div style="width: 100%; height: 10px"></div>
            <!-- 千支重计算 -->
            <vab-card>
              <template #header>
                <div class="card-header">
                  <div>千支重计算</div>
                  <el-tooltip effect="light" :offset="0" placement="left">
                    <template #content>
                      单个螺纹的截面应力参数，和螺纹的旋合长度无关。
                      <br />
                      螺纹应力截面积是指在螺纹连接中，由于螺纹连接中的应力分布是不均匀的，
                      <br />
                      螺纹内部所受到的力在其截面上所产生的应力所对应的截面面积。
                      <br />
                      由于螺纹连接中的应力分布是不均匀的，
                      <br />
                      因此在计算螺纹连接中的强度时，
                      <br />
                      需要考虑螺纹应力截面积这个参数。
                    </template>
                    <el-icon>
                      <info-filled />
                    </el-icon>
                  </el-tooltip>
                </div>
              </template>
              <el-row :gutter="25">
                <el-col :lg="6" :md="12" :sm="12" :xl="6" :xs="12">
                  <el-form label-width="45px">
                    <el-form-item>
                      <template #label>
                        <div>材料: &nbsp;</div>
                      </template>
                      <el-select v-model="queryForm.material" placeholder="材料">
                        <el-option v-for="item in dropDownDataSet.materialArr" :key="item.value" :label="item.label" :value="item.value" />
                      </el-select>
                    </el-form-item>
                  </el-form>
                </el-col>
                <el-col :lg="6" :md="12" :sm="12" :xl="6" :xs="12">
                  <el-form label-width="90px">
                    <el-form-item>
                      <template #label>
                        <div>千支重(kg):&nbsp;</div>
                      </template>
                      <el-input v-model="queryForm.weight" />
                    </el-form-item>
                  </el-form>
                </el-col>
                <el-col :lg="6" :md="12" :sm="12" :xl="6" :xs="12">
                  <el-form label-width="90px">
                    <el-form-item>
                      <template #label>
                        <div>单价(元/kg):&nbsp;</div>
                      </template>
                      <el-input v-model="queryForm.price" />
                    </el-form-item>
                  </el-form>
                </el-col>
                <el-col :lg="6" :md="12" :sm="12" :xl="6" :xs="12">
                  <el-form>
                    <el-form-item>
                      <el-button type="primary" @click="countNum('1')">计算</el-button>
                      <el-button @click="clearNum('1')">清除</el-button>
                    </el-form-item>
                  </el-form>
                </el-col>
              </el-row>
              <el-row :gutter="25" style="margin-top: 40px">
                <el-col :lg="8" :md="12" :sm="12" :xl="8" :xs="12">
                  <el-form label-width="120px">
                    <el-form-item>
                      <template #label>
                        <template v-if="dropDownDataControl.customize1 === 'mm'">公制(mm)</template>
                        <template v-else>美制(inch)</template>
                        结果:&nbsp;&nbsp;
                      </template>
                      <el-text style="font-size: 28px; font-weight: bold" type="primary">
                        {{ dropDownDataControl[dropDownDataControl.customize1 as DropDownDataControlKey].thousandWeight }}
                      </el-text>
                      <el-icon
                        v-show="dropDownDataControl[dropDownDataControl.customize1 as DropDownDataControlKey].thousandWeight"
                        color="#052965"
                        size="32"
                        style="margin-left: 10px; cursor: pointer"
                        title="复制"
                        @click="
                          handleCopy(
                            dropDownDataControl[dropDownDataControl.customize1 as DropDownDataControlKey].thousandWeight,
                            '千支重计算'
                          )
                        "
                      >
                        <document />
                      </el-icon>
                    </el-form-item>
                  </el-form>
                </el-col>
                <el-col :lg="6" :md="12" :sm="12" :xl="6" :xs="12">
                  <el-form label-width="80px">
                    <el-form-item>
                      <template #label>
                        <div>结果换算:&nbsp;</div>
                      </template>
                      <el-switch
                        v-model="dropDownDataControl.customize1"
                        active-text="公制(mm)"
                        active-value="mm"
                        inactive-text="美制(inch)"
                        inactive-value="inch"
                        inline-prompt
                        style="--el-switch-on-color: var(--default-color); --el-switch-off-color: #828282; height: 30px"
                        width="80"
                        @change="(value) => changeResult(value, '1')"
                      />
                    </el-form-item>
                  </el-form>
                </el-col>
              </el-row>
            </vab-card>
            <!-- 螺纹应力截面积 -->
            <div style="width: 100%; height: 10px"></div>
            <vab-card>
              <template #header>
                <div class="card-header">
                  <div>螺纹应力截面积</div>
                  <el-tooltip effect="light" :offset="0" placement="left">
                    <template #content>
                      单个螺纹的截面应力参数，和螺纹的旋合长度无关。
                      <br />
                      螺纹应力截面积是指在螺纹连接中，由于螺纹连接中的应力分布是不均匀的，
                      <br />
                      螺纹内部所受到的力在其截面上所产生的应力所对应的截面面积。
                      <br />
                      由于螺纹连接中的应力分布是不均匀的，
                      <br />
                      因此在计算螺纹连接中的强度时，
                      <br />
                      需要考虑螺纹应力截面积这个参数。
                    </template>
                    <el-icon>
                      <info-filled />
                    </el-icon>
                  </el-tooltip>
                </div>
              </template>
              <el-row :gutter="25" style="margin-top: 30px">
                <el-col :lg="8" :md="12" :sm="12" :xl="8" :xs="12">
                  <el-form label-width="120px">
                    <el-form-item>
                      <template #label>
                        <template v-if="dropDownDataControl.customize2 === 'mm'">公制(mm)</template>
                        <template v-else>美制(inch)</template>
                        结果:&nbsp;&nbsp;
                      </template>
                      <el-text style="font-size: 28px; font-weight: bold" type="primary">
                        {{ dropDownDataControl[dropDownDataControl.customize2 as DropDownDataControlKey].threadStressArea }}
                      </el-text>
                      <el-icon
                        v-show="dropDownDataControl[dropDownDataControl.customize2 as DropDownDataControlKey].threadStressArea"
                        color="#052965"
                        size="32"
                        style="margin-left: 10px; cursor: pointer"
                        title="复制"
                        @click="
                          handleCopy(
                            dropDownDataControl[dropDownDataControl.customize2 as DropDownDataControlKey].threadStressArea,
                            '螺纹应力截面积'
                          )
                        "
                      >
                        <document />
                      </el-icon>
                    </el-form-item>
                  </el-form>
                </el-col>
                <el-col :lg="6" :md="12" :sm="12" :xl="6" :xs="12">
                  <el-form label-width="80px">
                    <el-form-item>
                      <template #label>
                        <div>结果换算:&nbsp;</div>
                      </template>
                      <el-switch
                        v-model="dropDownDataControl.customize2"
                        active-text="公制(mm)"
                        active-value="mm"
                        inactive-text="美制(inch)"
                        inactive-value="inch"
                        inline-prompt
                        style="--el-switch-on-color: var(--default-color); --el-switch-off-color: #828282; height: 30px"
                        width="80"
                        @change="(value) => changeResult(value, '2')"
                      />
                    </el-form-item>
                  </el-form>
                </el-col>
              </el-row>
            </vab-card>
            <!-- 螺纹极限尺寸 -->
            <div style="width: 100%; height: 10px"></div>
            <vab-card>
              <template #header>
                <div class="card-header">
                  <div>螺纹极限尺寸</div>
                  <el-tooltip effect="light" :offset="0" placement="left">
                    <template #content>
                      单个螺纹的截面应力参数，和螺纹的旋合长度无关。
                      <br />
                      螺纹应力截面积是指在螺纹连接中，由于螺纹连接中的应力分布是不均匀的，
                      <br />
                      螺纹内部所受到的力在其截面上所产生的应力所对应的截面面积。
                      <br />
                      由于螺纹连接中的应力分布是不均匀的，
                      <br />
                      因此在计算螺纹连接中的强度时，
                      <br />
                      需要考虑螺纹应力截面积这个参数。
                    </template>
                    <el-icon>
                      <info-filled />
                    </el-icon>
                  </el-tooltip>
                </div>
              </template>
              <el-row :gutter="25">
                <el-col :lg="6" :md="12" :sm="12" :xl="6" :xs="12">
                  <el-form label-width="80px">
                    <el-form-item>
                      <template #label>
                        <div>螺纹类型:&nbsp;</div>
                      </template>
                      <el-select v-model="queryForm.thread" filterable placeholder="螺纹类型">
                        <el-option v-for="item in dropDownDataSet.threadArr" :key="item.value" :label="item.label" :value="item.value" />
                      </el-select>
                    </el-form-item>
                  </el-form>
                </el-col>
                <el-col :lg="6" :md="12" :sm="12" :xl="6" :xs="12">
                  <el-form label-width="80px">
                    <el-form-item>
                      <template #label>
                        <div>公差等级:&nbsp;</div>
                      </template>
                      <el-select v-model="queryForm.public" filterable placeholder="公差等级">
                        <el-option v-for="item in dropDownDataSet.publicArr" :key="item.value" :label="item.label" :value="item.value" />
                      </el-select>
                    </el-form-item>
                  </el-form>
                </el-col>
                <el-col :lg="6" :md="12" :sm="12" :xl="6" :xs="12">
                  <el-form>
                    <el-form-item>
                      <el-button type="primary" @click="countNum('3')">计算</el-button>
                      <el-button @click="clearNum('3')">清除</el-button>
                    </el-form-item>
                  </el-form>
                </el-col>
              </el-row>
              <el-row :gutter="25" style="margin-top: 40px">
                <el-col :lg="8" :md="12" :sm="12" :xl="8" :xs="12">
                  <el-form label-width="120px">
                    <el-form-item>
                      <template #label>
                        <template v-if="dropDownDataControl.customize3 === 'mm'">公制(mm)</template>
                        <template v-else>美制(inch)</template>
                        结果:&nbsp;&nbsp;
                      </template>
                      <el-text style="font-size: 28px; font-weight: bold" type="primary">
                        {{ dropDownDataControl[dropDownDataControl.customize3 as DropDownDataControlKey].threadLimitSize }}
                      </el-text>
                      <el-icon
                        v-show="dropDownDataControl[dropDownDataControl.customize3 as DropDownDataControlKey].threadLimitSize"
                        color="#052965"
                        size="32"
                        style="margin-left: 10px; cursor: pointer"
                        title="复制"
                        @click="
                          handleCopy(
                            dropDownDataControl[dropDownDataControl.customize3 as DropDownDataControlKey].threadLimitSize,
                            '螺纹极限尺寸'
                          )
                        "
                      >
                        <document />
                      </el-icon>
                    </el-form-item>
                  </el-form>
                </el-col>
                <el-col :lg="6" :md="12" :sm="12" :xl="6" :xs="12">
                  <el-form label-width="80px">
                    <el-form-item>
                      <template #label>
                        <div>结果换算:&nbsp;</div>
                      </template>
                      <el-switch
                        v-model="dropDownDataControl.customize3"
                        active-text="公制(mm)"
                        active-value="mm"
                        inactive-text="美制(inch)"
                        inactive-value="inch"
                        inline-prompt
                        style="--el-switch-on-color: var(--default-color); --el-switch-off-color: #828282; height: 30px"
                        width="80"
                        @change="(value) => changeResult(value, '3')"
                      />
                    </el-form-item>
                  </el-form>
                </el-col>
              </el-row>
            </vab-card>
            <!-- 最小破坏扭矩 -->
            <div style="width: 100%; height: 10px"></div>
            <vab-card>
              <template #header>
                <div class="card-header">
                  <div>最小破坏扭矩</div>
                  <el-tooltip effect="light" :offset="0" placement="left">
                    <template #content>
                      单个螺纹的截面应力参数，和螺纹的旋合长度无关。
                      <br />
                      螺纹应力截面积是指在螺纹连接中，由于螺纹连接中的应力分布是不均匀的，
                      <br />
                      螺纹内部所受到的力在其截面上所产生的应力所对应的截面面积。
                      <br />
                      由于螺纹连接中的应力分布是不均匀的，
                      <br />
                      因此在计算螺纹连接中的强度时，
                      <br />
                      需要考虑螺纹应力截面积这个参数。
                    </template>
                    <el-icon>
                      <info-filled />
                    </el-icon>
                  </el-tooltip>
                </div>
              </template>
              <el-row :gutter="25">
                <el-col :lg="6" :md="12" :sm="12" :xl="6" :xs="12">
                  <el-form label-width="50px">
                    <el-form-item>
                      <template #label>
                        <div>强度:&nbsp;</div>
                      </template>
                      <el-select v-model="queryForm.strength" filterable placeholder="强度">
                        <el-option v-for="item in dropDownDataSet.strengthArr" :key="item.value" :label="item.label" :value="item.value" />
                      </el-select>
                    </el-form-item>
                  </el-form>
                </el-col>
                <el-col :lg="6" :md="12" :sm="12" :xl="6" :xs="12">
                  <el-form>
                    <el-form-item>
                      <el-button type="primary" @click="countNum('4')">计算</el-button>
                      <el-button @click="clearNum('4')">清除</el-button>
                    </el-form-item>
                  </el-form>
                </el-col>
              </el-row>
              <el-row :gutter="25" style="margin-top: 40px">
                <el-col :lg="8" :md="12" :sm="12" :xl="8" :xs="12">
                  <el-form label-width="120px">
                    <el-form-item>
                      <template #label>
                        <template v-if="dropDownDataControl.customize4 === 'mm'">公制(mm)</template>
                        <template v-else>美制(inch)</template>
                        结果:&nbsp;&nbsp;
                      </template>
                      <el-text style="font-size: 28px; font-weight: bold" type="primary">
                        {{ dropDownDataControl[dropDownDataControl.customize4 as DropDownDataControlKey].minimumTorqueVal }}
                      </el-text>
                      <el-icon
                        v-show="dropDownDataControl[dropDownDataControl.customize4 as DropDownDataControlKey].minimumTorqueVal"
                        color="#052965"
                        size="32"
                        style="margin-left: 10px; cursor: pointer"
                        title="复制"
                        @click="
                          handleCopy(
                            dropDownDataControl[dropDownDataControl.customize4 as DropDownDataControlKey].minimumTorqueVal,
                            '最小破坏扭矩'
                          )
                        "
                      >
                        <document />
                      </el-icon>
                    </el-form-item>
                  </el-form>
                </el-col>
                <el-col :lg="6" :md="12" :sm="12" :xl="6" :xs="12">
                  <el-form label-width="80px">
                    <el-form-item>
                      <template #label>
                        <div>结果换算:&nbsp;</div>
                      </template>
                      <el-switch
                        v-model="dropDownDataControl.customize4"
                        active-text="公制(mm)"
                        active-value="mm"
                        inactive-text="美制(inch)"
                        inactive-value="inch"
                        inline-prompt
                        style="--el-switch-on-color: var(--default-color); --el-switch-off-color: #828282; height: 30px"
                        width="80"
                        @change="(value) => changeResult(value, '4')"
                      />
                    </el-form-item>
                  </el-form>
                </el-col>
              </el-row>
            </vab-card>
            <div style="width: 100%; height: 100px"></div>
          </div>
          <!-- 标准解读 -->
          <div v-if="sideNavigationTabsActive == 'Standard'">
            <standard-viewer :content="content" />
          </div>
          <!-- 供应商 -->
          <div v-if="sideNavigationTabsActive == 'Supplier'">
            <supplier-viewer />
          </div>
        </el-col>
      </el-row>
    </section>
  </el-drawer>
</template>
<script setup lang="ts">
import { Document, QuestionFilled, InfoFilled } from '@element-plus/icons-vue'
import { productsDetail } from '/@/api/products'
import diameterImg from '/@/assets/diameter.png'
import lengthImg from '/@/assets/length.png'
import pitchImg from '/@/assets/pitch.png'
import clip from '/@/utils/clipboard'
const templateName = 'ProductsDetail'
defineOptions({
  name: templateName,
})
const emit = defineEmits(['handle-delete'])
const state = reactive<any>({
  drawerdFormVisible: false,
})
const config = reactive({
  loading: false,
  title: '',
})
// 筛选值集合
const queryForm = reactive({
  diameter: '', // 直径
  length: '', // 长度
  pitch: '', // 螺距
  material: '', // 材料
  weight: '', // 千支重
  price: '', // 价格
  thread: '', // 螺纹类型
  public: '', // 公差等级
  strength: '', // 强度
})
// 下拉数据集合
const dropDownDataSet = reactive({
  // 直径下拉
  diameterArr: [
    {
      value: 'M1.6',
      label: 'M1.6',
    },
    {
      value: 'M2',
      label: 'M2',
    },
    {
      value: 'M2.5',
      label: 'M2.5',
    },
    {
      value: 'M3',
      label: 'M3',
    },
    {
      value: 'M3.5',
      label: 'M3.5',
    },
    {
      value: 'M4',
      label: 'M4',
    },
    {
      value: 'M5',
      label: 'M5',
    },
    {
      value: 'M6',
      label: 'M6',
    },
    {
      value: 'M8',
      label: 'M8',
    },
    {
      value: 'M10',
      label: 'M10',
    },
    {
      value: 'M12',
      label: 'M12',
    },
  ],
  // 长度下拉
  lengthArr: [
    {
      value: '4',
      label: '4',
    },
    {
      value: '5',
      label: '5',
    },
    {
      value: '6',
      label: '6',
    },
    {
      value: '8',
      label: '8',
    },
    {
      value: '10',
      label: '10',
    },
    {
      value: '12',
      label: '12',
    },
    {
      value: '16',
      label: '16',
    },
    {
      value: '18',
      label: '18',
    },
  ],
  // 螺距下拉
  pitchArr: [
    {
      value: '0.35',
      label: '0.35',
    },
    {
      value: '4',
      label: '4',
    },
    {
      value: '5',
      label: '5',
    },
  ],
  // 材料下拉
  materialArr: [
    {
      value: '1',
      label: '碳钢',
    },
    {
      value: '2',
      label: '不锈钢',
    },
    {
      value: '3',
      label: '铜合金',
    },
    {
      value: '4',
      label: '铝合金',
    },
    {
      value: '5',
      label: '钛合金',
    },
  ],
  // 强度
  strengthArr: [
    {
      value: '8.8',
      label: '8.8级',
    },
    {
      value: '9.8',
      label: '9.8级',
    },
    {
      value: '10.8',
      label: '10.8级',
    },
    {
      value: '12.9',
      label: '12.9级',
    },
  ],
  // 公差等级
  publicArr: [
    {
      value: '4e',
      label: '4e',
    },
    {
      value: '4f',
      label: '4f',
    },
    {
      value: '4g',
      label: '4g',
    },
    {
      value: '4h',
      label: '4h',
    },
    {
      value: '6e',
      label: '6e',
    },
    {
      value: '6f',
      label: '6f',
    },
    {
      value: '6h',
      label: '6h',
    },
  ],
  // 螺纹类型
  threadArr: [
    {
      value: '1',
      label: '外螺纹',
    },
    {
      value: '2',
      label: '内螺纹',
    },
  ],
})
type DropDownDataControlKey = 'mm' | 'inch'
const dropDownDataControl = reactive<{
  customize1: DropDownDataControlKey
  customize2: DropDownDataControlKey
  customize3: DropDownDataControlKey
  customize4: DropDownDataControlKey
  mm: {
    thousandWeight: string
    threadStressArea: string
    threadLimitSize: string
    minimumTorqueVal: string
  }
  inch: {
    thousandWeight: string
    threadStressArea: string
    threadLimitSize: string
    minimumTorqueVal: string
  }
}>({
  customize1: 'mm',
  customize2: 'mm',
  customize3: 'mm',
  customize4: 'mm',
  mm: {
    thousandWeight: '12.96',
    threadStressArea: '54.06',
    threadLimitSize: '32.87',
    minimumTorqueVal: '29.51',
  },
  inch: {
    thousandWeight: '43.69',
    threadStressArea: '67.24',
    threadLimitSize: '7.05',
    minimumTorqueVal: '8.64',
  },
})
// 直径改变
const handleDiameterChange = (value: any) => {
  queryForm.diameter = value
}
// 螺距改变
const handlePitchChange = (value: any) => {
  queryForm.pitch = value
}
const changeResult = (value: string | number | boolean, type: string) => {
  // 1: 千支重计算、2：螺纹应力截面积、3:螺纹极限尺寸、 4:螺纹极限尺寸
  console.log(value, type)
}
// 计算
const countNum = (type: string) => {
  // 1: 千支重计算、3:螺纹极限尺寸、 4:螺纹极限尺寸
}
// 清除
const clearNum = (type: string) => {
  // 1: 千支重计算、3:螺纹极限尺寸、 4:螺纹极限尺寸
  if (Number(type) === 1) dropDownDataControl[dropDownDataControl.customize1].thousandWeight = ''
  if (Number(type) === 3) dropDownDataControl[dropDownDataControl.customize3].threadLimitSize = ''
  if (Number(type) === 4) dropDownDataControl[dropDownDataControl.customize4].minimumTorqueVal = ''
}
// 复制
const handleCopy = (val: any, text: any) => {
  clip(val, text)
}
// 基本信息、标准解读 tab
const sideNavigationTabsActive = ref('Basic')
const sideNavigationTabsList = ref([
  {
    name: 'Basic',
    label: '基本信息',
  },
  {
    name: 'Standard',
    label: '标准解读',
  },
  {
    name: 'Supplier',
    label: '供应商',
  },
])
const changeLeftMenuTab = (value: string) => {
  sideNavigationTabsActive.value = value
}
// 尺寸图、三维图、CAD图tab
const imgTabsActive = ref('Size')
const imgTabsList = reactive([
  {
    name: 'Size',
    label: '尺寸图',
  },
  {
    name: '3D',
    label: '三维图',
  },
  {
    name: 'CAD',
    label: 'CAD图',
  },
])
const showView = async (row: any) => {
  if (row) {
    getProductsDetail(row.id)
  }
  open()
  setTimeout(() => {
    config.loading = false
  }, 500)
}
const open = () => {
  state.drawerdFormVisible = true
  config.loading = true
}
// 关闭
const handleClose = () => {
  close()
}
const close = () => {
  state.drawerdFormVisible = false
}
const getProductsDetail = async (id: number) => {
  const { data }: any = await productsDetail({ id: id })
}
const content = ref(`
<p>—GB/T  5786六角头螺栓细牙全螺纹。</p>
<p>本标准是“六角头螺栓”系列国家标准之一，该系列包括：</p>
<p>本标准按照GB/T  1.1—2009给出的规则起草。</p>
<p>本标准代替GB/T 5782—2000《六角头螺栓》，与GB/T 5782—2000相比，主要技术变化如下;</p>
<p>——删除“如需其他技术要求，...…GB/T 3098.6和GB/T 3103.1)中选择。”(2000年版第1章）;</p>
<p>——引用螺纹标准统一为GB/T 193、GB/T 9145（第2章）;</p>
<p>——仅对钢产品规定表面缺陷：GB/T  5779.1（表3);</p>
<p>——增加钢螺栓表面不经处理，删除氧化（表3);</p>
<p>——增加钢螺栓非电解锌片涂层技术要求按GB/T 5267.2（表3);</p>
<p>——增加不锈钢螺栓钝化处理技术要求按GB/T 5267.4（表3）;</p>
<p>——增加有色金属螺栓电锁技术要求按GB/T 5267.1;</p>
<p>规定标记中仅允许省略：表面不经处理(5.2)。</p>
<p>本标准使用重新起草法修改采用ISO 4014:2011《六角头螺栓	产品等级A和B级》（英文版）。</p>
<p>本标准与ISO 4014 :2011的技术性差异及其原因如下：</p>
<p>——删除ISO 4014规定：“如需其他技术要求，……ISO 4753和ISO 4759-1中选择。”（第1章），不属于本标准规定的内容；</p>
<p></p>
<p>—GB/T  5786六角头螺栓细牙全螺纹。</p>
<p>本标准是“六角头螺栓”系列国家标准之一，该系列包括：</p>
<p>本标准按照GB/T  1.1—2009给出的规则起草。</p>
<p>本标准代替GB/T 5782—2000《六角头螺栓》，与GB/T 5782—2000相比，主要技术变化如下;</p>
<p>——删除“如需其他技术要求，...…GB/T 3098.6和GB/T 3103.1)中选择。”(2000年版第1章）;</p>
<p>——引用螺纹标准统一为GB/T 193、GB/T 9145（第2章）;</p>
<p>——仅对钢产品规定表面缺陷：GB/T  5779.1（表3);</p>
<p>——增加钢螺栓表面不经处理，删除氧化（表3);</p>
<p>——增加钢螺栓非电解锌片涂层技术要求按GB/T 5267.2（表3);</p>
<p>——增加不锈钢螺栓钝化处理技术要求按GB/T 5267.4（表3）;</p>
<p>——增加有色金属螺栓电锁技术要求按GB/T 5267.1;</p>
<p>规定标记中仅允许省略：表面不经处理(5.2)。</p>
<p>本标准使用重新起草法修改采用ISO 4014:2011《六角头螺栓	产品等级A和B级》（英文版）。</p>
<p>本标准与ISO 4014 :2011的技术性差异及其原因如下：</p>
<p>——删除ISO 4014规定：“如需其他技术要求，……ISO 4753和ISO 4759-1中选择。”（第1章），不属于本标准规定的内容；</p>
<p>—GB/T  5786六角头螺栓细牙全螺纹。</p>
<p>本标准是“六角头螺栓”系列国家标准之一，该系列包括：</p>
<p>本标准按照GB/T  1.1—2009给出的规则起草。</p>
<p>本标准代替GB/T 5782—2000《六角头螺栓》，与GB/T 5782—2000相比，主要技术变化如下;</p>
<p>——删除“如需其他技术要求，...…GB/T 3098.6和GB/T 3103.1)中选择。”(2000年版第1章）;</p>
<p>——引用螺纹标准统一为GB/T 193、GB/T 9145（第2章）;</p>
<p>——仅对钢产品规定表面缺陷：GB/T  5779.1（表3);</p>
<p>——增加钢螺栓表面不经处理，删除氧化（表3);</p>
<p>——增加钢螺栓非电解锌片涂层技术要求按GB/T 5267.2（表3);</p>
<p>——增加不锈钢螺栓钝化处理技术要求按GB/T 5267.4（表3）;</p>
<p>——增加有色金属螺栓电锁技术要求按GB/T 5267.1;</p>
<p>规定标记中仅允许省略：表面不经处理(5.2)。</p>
<p>本标准使用重新起草法修改采用ISO 4014:2011《六角头螺栓	产品等级A和B级》（英文版）。</p>
<p>本标准与ISO 4014 :2011的技术性差异及其原因如下：</p>
<p>——删除ISO 4014规定：“如需其他技术要求，……ISO 4753和ISO 4759-1中选择。”（第1章），不属于本标准规定的内容；</p>
`)
defineExpose({
  showView,
})
</script>
<style lang="scss">
.el-overlay.products-detail-drawer {
  .vab-card .el-card__header {
    height: 40px;
    padding: 10px;
    font-size: 16px;
    background: rgba(5, 41, 101, 0.1);
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
  }

  .el-switch__core {
    height: 23px;
  }

  .under-tabs-select-box {
    width: 100%;
    padding-bottom: 10px;
    background: var(--default-color);

    .el-form-item__label {
      div {
        color: #fff;
      }
    }

    .under-tabs-img-box {
      display: flex;
      align-items: center;
      justify-content: center;
      width: 100%;
      height: 32px;
      background: #fff;
      border-right: 1px solid var(--default-color);
      border-top-left-radius: 3px;
      border-bottom-left-radius: 3px;

      div {
        font-size: 14px;
        color: var(--default-color);
      }

      img {
        width: auto;
        height: 23px;
        margin-left: 5px;
      }
    }

    .under-tabs-select-input {
      display: flex;
      flex: 0 0 auto;
      align-items: center;
      height: 40px;
      padding-top: 12px;
      line-height: 40px;

      .unit-text {
        margin-left: 6px;
        color: #fff;
        white-space: nowrap;
      }

      .total-value {
        width: calc(100% - 48px);
        height: 30px;
        padding: 0 5px;
        font-size: 15px;
        line-height: 30px;
        color: var(--default-color);
        background: #fff;
        border-radius: 4px;
      }
    }

    .notice-title {
      display: flex;
      flex-wrap: nowrap;
      align-items: center;
      justify-content: flex-end;
      width: 100%;
      padding-top: 15px;
      margin-left: 90px;

      .label-title {
        padding-left: 3px;
        color: #fff;
        white-space: nowrap;
      }
    }
  }

  .el-select__wrapper {
    border-radius: 3px !important;
  }

  .custom-el-form {
    .el-select__wrapper {
      border-top-left-radius: 0 !important;
      border-bottom-left-radius: 0 !important;
      box-shadow: none;
    }

    .el-form-item__label {
      padding: 0 !important;
    }

    .el-form-item {
      margin-bottom: 0 !important;
    }
  }

  .el-drawer__header {
    margin-bottom: 0 !important;
  }

  .products-detail-drawer-content {
    padding-bottom: 50px;
  }

  .products-detail-drawer-content,
  .el-row {
    width: 100%;
    height: 100%;
  }

  .el-tabs__item {
    justify-content: center !important;
  }

  .drawer-left-menu {
    width: 100%;

    div {
      width: 100%;
      height: 45px;
      line-height: 45px;
      text-align: center;

      &.active {
        color: #000;
        background: #eaf0f0;
        border-left: 3px solid var(--default-color);
      }
    }
  }

  h2.name-text {
    padding-bottom: 15px;
    font-size: 20px;
    color: var(--default-color);
  }

  .en-name {
    padding-bottom: 20px;
  }

  .tabs-label {
    font-size: 13px;
    font-weight: bold;
  }

  .el-tabs__item {
    width: 120px;
    padding: 0;

    &.is-active {
      background: #eaf0f0;
    }
  }

  .tabs-content {
    position: relative;
    width: 100%;
    height: 600px;
    padding: 10px;
    overflow: hidden;
    background: #eaf0f0;
    border: 1px solid var(--default-color);
  }
}
</style>
