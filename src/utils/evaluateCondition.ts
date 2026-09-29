import * as math from 'mathjs'
/**
 * 条件表达式解析器
 * @param {string} condition - 原始条件表达式
 * @returns {boolean} 条件计算结果
 * 实现逻辑：
 * 1. 构建变量上下文（包含parameters和表单值）
 * 2. 替换表达式中的运算符（≤→<=，≥→>=，&→&&，|→||）
 * 3. 通过eval执行表达式（注意安全风险）
 */
export const evaluateConditionDefault = (condition: string, parameters: any, queryForm: any): boolean => {
  const parsedCondition = evaluateConditionFormula(condition, parameters, queryForm)
  // console.log('parsedCondition:', `|${parsedCondition}|`)
  try {
    // return math.evaluate(parsedCondition, variables);
    return evaluateExpression(replaceFullWidthSymbols(parsedCondition)) // eval 收敛至 evaluateExpression 单点（白名单受控）
  } catch (error) {
    console.error('Error evaluating condition:', condition, error)
    return false
  }
}

export const toHalfWidth = (str: string) => {
  return str
    .replace(/[\uff10-\uff5e]/g, function (char) {
      return String.fromCharCode(char.charCodeAt(0) - 65248)
    })
    .replace(/\u3000/g, ' ') // 全角空格转换为半角空格
}

/**
 * 区间键形态规范化（GB 表格直录/历史数据容错）
 * 兼容形态：a~b / a～b / ~b / a~ / >50 / ≥3 / >3<L&L≤6 / ≤3 / <3 / 裸数字
 * 语义等价改写为 mathjs 可求值的双端/单端比较，不改动含字母的完整条件
 */
export const normalizeConditionKey = (condition: string): string => {
  let c = (condition || '').trim()
  // 统一全角比较前缀为半角（≥3 → >=3，＞3 → >3，≤3 → <=3，＜3 → <3）
  c = c.replace(/^≥/, '>=').replace(/^＞/, '>').replace(/^≤/, '<=').replace(/^＜/, '<')
  // 前缀大于号形态：>50 → 50<L；>3<L&L≤6 → 3<L&L≤6；>3.5~6 → 递归处理残余区间
  const prefixGt = /^[>]\s*([\d.]+)(.*)$/.exec(c)
  if (prefixGt) {
    const [, num, rest] = prefixGt
    if (!rest.trim()) return `${num}<L`
    return normalizeConditionKey(`${num}${rest}`)
  }
  // a~b 双端区间（~ / ～ 已覆盖全角；容忍空格）
  const tilde = /^([\d.]*)\s*[~～]\s*([\d.]*)$/.exec(c)
  if (tilde) {
    const [, lo, hi] = tilde
    if (lo && hi) return `${lo}≤L&L≤${hi}` // 闭区间保守处理，防漏档
    if (hi) return `L≤${hi}`
    if (lo) return `${lo}<L`
  }
  // 剩余无字母的裸条件补上比较主体 L
  if (!/[a-zA-Z]/.test(c)) {
    if (/^<=?/.test(c)) return `L${c}` // <=3 → L<=3；<3 → L<3
    if (/^>=?/.test(c)) {
      // >=3 → 3<=L；>3 → 3<L
      const op = c.startsWith('>=') ? '>=' : '>'
      const num = c.replace(/^>=?/, '')
      return `${num}${op === '>' ? '<' : '<='}L`
    }
    if (/^[\d.]+$/.test(c)) return `L≤${c}` // 裸数字（历史异常）→ 上限
  }
  return c
}

export const evaluateConditionFormula = (condition: string, parameters: any, queryForm: any) => {
  const monParams = parameters?.[queryForm?.mon]
  const variables = Object.fromEntries(
    Object.entries(monParams || {})
      .filter(([key, value]) => typeof value === 'string' || typeof value === 'number')
      .map(([key, value]) => [key, parseFloat(String(value)) || value]) // 转换为数字（如果可能）
  )
  variables['L'] = queryForm.diameterLength || 0
  variables['P'] = queryForm.pitch || variables['P'] || 0
  // const { L, d } = variables;
  const parsedCondition = normalizeConditionKey(condition)
    .replace(/≤/g, '<=')
    .replace(/≥/g, '>=')
    .replace(/&/g, '&&')
    .replace(/\|/g, '||')
    .replace(/./g, (char) => toHalfWidth(char))
    .replace(/([a-zA-Z]+)/g, (match) => {
      // 变量未收录（键含非参数字母或 mon 为空）时回退 0，避免拼出 undefined 导致整体求值失败
      const value = variables[match]
      return value === undefined || value === null || Number.isNaN(Number(value)) ? '0' : String(value)
    })

  return parsedCondition
}
/**
 * 全角转半角
 * @param str
 * @returns
 */
export const replaceFullWidthSymbols = (str: string): string => {
  const symbolMap: { [key: string]: string } = {
    '（': '(',
    '）': ')',
    '［': '[',
    '］': ']',
    '｛': '{',
    '｝': '}',
    '＋': '+',
    '－': '-',
    '＊': '*',
    '／': '/',
    '＝': '=',
    '＜': '<',
    '＞': '>',
    '％': '%',
    '＆': '&',
    '：': ':',
    '．': '.',
    '，': ',',
    '＇': "'",
    '＂': '"',
    '　': ' ', // 全角空格
    '！': '!',
    '？': '?',
    '～': '~',
    '｜': '|',
    '＼': '\\',
    '＄': '$',
    '＾': '^',
    '＃': '#',
    '＠': '@',
    '｀': '`',
    '￥': '\\',
    '…': '...',
    '——': '-',
    '‘': "'",
    '’': "'",
    '“': '"',
    '”': '"',
    '《': '<',
    '》': '>',
    '；': ';',
    '【': '[',
    '】': ']',
    // 可以继续添加你需要处理的符号
  }
  // 构建正则表达式：匹配所有 symbolMap 中的键
  const pattern = new RegExp(`[${Object.keys(symbolMap).join('')}]`, 'g')
  return str.replace(pattern, (match) => symbolMap[match])
}

/**
 * 受控布尔表达式求值（eval 收敛单点）
 * 仅放行「数字/空白/比较与逻辑运算符」，拒绝任何字母、引号、分号等可执行 JS 片段；
 * 表达式非法或超长时直接返回 false，杜绝任意代码执行。
 */
export const evaluateExpression = (expr: string): boolean => {
  if (typeof expr !== 'string' || expr.length > 512) {
    return false
  }
  if (!/^[\d\s.+\-*/%<>=!&|()]*$/.test(expr)) {
    console.error('[evaluateExpression] 拒绝执行非白名单表达式:', expr)
    return false
  }
  try {
    // C-B2: 用 mathjs 受控求值替代 eval，杜绝任意代码执行
    // mathjs 不识别 JS 布尔运算符，先翻译 &&/||/! -> and/or/not
    const mathExpr = expr
      .replace(/&&/g, ' and ')
      .replace(/\|\|/g, ' or ')
      .replace(/!(?!=)/g, ' not ')
    return !!math.evaluate(mathExpr, {})
  } catch (error) {
    console.error('[evaluateExpression] 表达式计算失败:', expr, error)
    return false
  }
}

/**
 * 解析并展开嵌套公式，支持变量替换和条件表达式（如 a != null ? a : b）
 *
 * @param target - 要解析的目标公式字符串，如 "(Vddlj + VdP)*10"
 * @param formulas - 变量名到表达式的映射表，如 { sAvg: "(sMax + sMin)/2", Vddlj: "0.823*sAvg^2*kAvg" }
 * @param values - 实际数值变量表（不会被展开），如 { sMax: 2.0, sMin: 1.0 }
 * @returns 展开后的纯表达式字符串，可直接用于 mathjs 计算
 */
export function resolveFormula(target: string, formulas: Record<string, string>, values: Record<string, any> = {}): string {
  const cache: Record<string, string> = {} // 缓存已展开的中间表达式
  function expand(expr: string): string {
    // 替换 π -> pi，^ -> **（mathjs 兼容）
    expr = expr.replace(/π/gi, 'pi') //.replace(/\^/g, '**');
    const varRegex = /([a-zA-Z][a-zA-Z0-9_]*)/g
    let match: RegExpExecArray | null
    const dependencies = new Set<string>()
    // 找出所有需要展开的变量（存在于 formulas 且不在 values 中）
    while ((match = varRegex.exec(expr)) !== null) {
      const varName = match[1]
      if (formulas[varName] && !(varName in values)) {
        dependencies.add(varName)
      }
    }
    // 递归展开每个依赖项
    for (const dep of dependencies) {
      if (!cache[dep]) {
        cache[dep] = expand(formulas[dep])
      }
      // 使用词边界替换，避免部分匹配（如 kAvg 不匹配 dkAvg）
      const regex = new RegExp(`\\b${dep}\\b`, 'g')
      // expr = expr.replace(regex, `(${cache[dep]})`);
      // console.log(`regex[${dep}]:`, cache[dep]);
      // 依赖表达式求值失败时（未知符号，如产品公式引用了未配置的 whorlStress/σbMin），
      // 降级为保留展开式而非抛错，交由外层 evaluateFormula/math.evaluate 兜底（对齐 v1.0.0 行为）
      try {
        expr = expr.replace(regex, `${math.evaluate(cache[dep], values)}`)
      } catch (error: any) {
        console.warn(`[resolveFormula] 依赖 ${dep} 求值失败，保留展开式:`, error?.message)
        expr = expr.replace(regex, `(${cache[dep]})`)
      }
    }
    return expr
  }
  return expand(target)
}

export const evaluateFormula = (columnar: string, formulas: any, variables: any) => {
  /**
   * 使用公式表批量替换目标字符串中的公式标识符
   * @param acc 累加器 - 存储当前替换结果的中间字符串
   * @param key 当前正在处理的公式标识符
   * @param formulas 公式映射表 - 包含需要替换的标识符与对应公式的键值对
   * @param columnar 初始列式字符串 - 需要执行替换操作的原始字符串
   * @returns 完成所有公式标识符替换后的最终字符串
   *
   * 实现逻辑：
   * 通过正则表达式全局匹配，将初始字符串中所有出现的公式标识符（formulas对象的键）
   * 替换为对应的公式表达式（formulas对象的值），最终生成完整的列式计算公式字符串
   */
  let columnarFormulas = resolveFormula(replaceFullWidthSymbols(columnar), formulas, variables)
  // resolveFormula 抛错时降级为原始列式，交由下方 math.evaluate 统一兜底返回 0，避免异常穿透调用方
  if (!columnarFormulas || typeof columnarFormulas !== 'string') {
    columnarFormulas = replaceFullWidthSymbols(columnar)
  }
  // 替换 π 为 math.pi
  columnarFormulas = columnarFormulas.replace(/π/gi, 'pi')
  // 替换 ^ 为幂运算符 pow
  // columnarFormulas = columnarFormulas.replace(/\^/g, '**');
  // console.log('columnar', columnar);
  // console.log('formulas', JSON.stringify(formulas));
  // console.log("columnarFormulas", columnarFormulas, JSON.stringify(variables));
  // console.log("math", math.evaluate(columnarFormulas, variables));

  try {
    return math.evaluate(columnarFormulas, variables)
  } catch (error: any) {
    return 0
  }
}

export const getMathEvaluate = (formulas: any, variables: any) => {
  // console.log("variables", variables);
  // 替换 π 为 math.pi
  let columnarFormulas = replaceFullWidthSymbols(formulas).replace(/π/gi, 'pi')
  // 替换 ^ 为幂运算符 pow
  columnarFormulas = columnarFormulas.replace(/π/gi, 'pi')
  // columnarFormulas = formulas.replace(/\^/g, '**');
  // console.log('columnarFormulas:', formulas, JSON.stringify(variables));
  try {
    return math.evaluate(columnarFormulas, variables)
  } catch (error: any) {
    return 0
  }
}

export const getVariables = (parameters: any, tolerance: any, queryForm: any) => {
  const myL = queryForm.diameterLength || 0
  const variables = {} as any
  let conditionL = 'A'
  variables['L'] = Number(queryForm.diameterLength || 0)
  variables['P'] = Number(queryForm.pitch || variables['P'] || 0)
  // 2. 替换占位符 {xxx} 为 值
  if (queryForm.mon && parameters[queryForm.mon]) {
    // 替换逻辑
    for (const [key, value] of Object.entries(parameters[queryForm.mon])) {
      if (typeof value === 'string' || typeof value === 'number') {
        if (key === 'L') {
          variables[key] = Number(myL)
        } else if (key === 'P') {
          variables[key] = Number(queryForm.pitch || value || 0)
        } else {
          variables[key] = Number.isNaN(value) ? Number(value || 0) : value
        }
      } else if (typeof value === 'object' && value !== null) {
        // 复杂替换：对象类型
        if ('condition' in value) {
          // 根据条件选择 A 或 B
          const conditionValue = value as any
          const conditionResult = evaluateConditionDefault(conditionValue.condition, parameters, queryForm)
          // 存在完整 A/B 的情况（A/B 均为非空值才算完整，对齐 v1.0.0 truthiness 语义。
          // 升级时误改为 hasOwnProperty，导致 A 为空串的产品（如 M27/M30 大直径档）
          // 被错判为完整 AB：condition 求值为假时取 B 档宽公差，档位选取回归）
          if (conditionValue.A && conditionValue.B) {
            conditionL = conditionResult ? 'A' : 'B'
            variables[key] = Number(conditionResult ? (conditionValue.A ?? 0) : (conditionValue.B ?? 0))
          } else if (conditionResult) {
            // AB不完整的情况，判断值是否符合条件；对齐 v1.0.0：任意子键求值为真即可改写 conditionL
            Object.keys(conditionValue).forEach((_key: string) => {
              const key2 = _key as string
              if (evaluateConditionDefault(key2, parameters, queryForm)) {
                conditionL = key2
              }
            })
          }
        } else {
          // 根据范围选择值
          let replacementValue = ''
          for (const [range, rangeValue] of Object.entries(value)) {
            if (evaluateConditionDefault(range, parameters, queryForm)) {
              replacementValue = rangeValue
              if (!!rangeValue) break
            }
          }
          variables[key] = Number.isNaN(replacementValue) ? Number(replacementValue) : replacementValue
        }
      }
    }
  }
  function calculateExpression(expression: string) {
    try {
      // 使用 evaluate 方法计算表达式的值
      return math.evaluate(expression)
    } catch (error: any) {
      console.error(`计算过程中出现错误: ${error.message}`)
      return null
    }
  }
  // 计算公差（恢复 v1.0.0 三分支语义：conditionL 档优先 → 单键 A/B/C → 遍历子键条件命中；
  // 保留偏差值形态规范化与求值失败跳过防御，不回退 eval，保住安全修复）
  if (tolerance) {
    /**
     * 偏差值形态规范化：兼容全角 －/＋、±x 单值、范围 "-0.2~+0.2"
     */
    const normalizeToleranceValue = (value: any, isMax: boolean): string => {
      if (value === null || value === undefined || value === '') {
        return isMax ? '+0' : '-0'
      }
      const s = replaceFullWidthSymbols(toHalfWidth(String(value))).trim()
      if (s.includes('±')) {
        const v = s.replace(/±/g, '').trim()
        return isMax ? `+${v.replace(/^\+/, '')}` : `-${v.replace(/^-/, '')}`
      }
      if (s.includes('~') || s.includes('～')) {
        const parts = s.split(/[~～]/)
        const pick = isMax ? parts[1] : parts[0]
        return pick === undefined || pick.trim() === '' ? (isMax ? '+0' : '-0') : pick.trim()
      }
      return s
    }
    const writeLMinMax = (rawMin: any, rawMax: any, item: string, grade: string) => {
      const LMin = normalizeToleranceValue(rawMin, false)
      const LMax = normalizeToleranceValue(rawMax, true)
      const minRes = calculateExpression(`${myL}${LMin.startsWith('+') ? '' : '+'}${LMin}`)
      const maxRes = calculateExpression(`${myL}${LMax.startsWith('+') ? '' : '+'}${LMax}`)
      if (minRes === null || maxRes === null) {
        console.warn(`[getVariables] 公差偏差值求值失败，跳过该级: item=${item} grade=${grade} min=${rawMin} max=${rawMax}`)
        return
      }
      variables['LMin'] = formatNumber(minRes)
      variables['LMax'] = formatNumber(maxRes)
    }
    if (!tolerance || typeof tolerance !== 'object') return variables
    Object.keys(tolerance).map((item: string) => {
      const parsedCondition = evaluateConditionDefault(item, parameters, queryForm)
      if (parsedCondition) {
        const grades = tolerance[item]
        if (!grades || typeof grades !== 'object') return
        if (grades[conditionL]) {
          // 分支一：目标公差级（conditionL 命中的档位）优先
          const LMax = grades[conditionL]['max'] || '-0'
          const LMin = grades[conditionL]['min'] || '+0'
          writeLMinMax(LMin, LMax, item, conditionL)
        } else if (Object.keys(grades).length === 1 && ['A', 'B', 'C'].includes(Object.keys(grades)[0])) {
          // 分支二：仅单键且为 A/B/C 公差级
          const key0 = Object.keys(grades)[0]
          const LMax = grades[key0]['max'] || '-0'
          const LMin = grades[key0]['min'] || '+0'
          writeLMinMax(LMin, LMax, item, key0)
        } else {
          // 分支三：遍历全部子键，条件命中即写入（对齐 v1.0.0 行为）
          Object.keys(grades).map((k: string) => {
            if (evaluateConditionDefault(k, parameters, queryForm)) {
              const LMax = grades[k]['max'] || '-0'
              const LMin = grades[k]['min'] || '+0'
              writeLMinMax(LMin, LMax, item, k)
            }
          })
        }
      }
    })
  }
  return variables
}

export const formatNumber = (num: number | string, fixed: number = 3) => {
  return Number(Number(num || 0).toFixed(fixed || 3)).toString()
}
