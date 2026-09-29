/***
 * @Author: trexwb
 * @Date: 2025-03-31 14:41:57
 * @LastEditors: trexwb
 * @LastEditTime: 2025-03-31 14:41:59
 * @FilePath: /client/console/src/utils/parseExcelToJSON.js
 * @Description:
 * @一花一世界，一叶一如来
 * @Copyright (c) 2025 by 杭州大美, All Rights Reserved.
 */
// 全角转半角
function toHalfWidth(str: string) {
  return str
    .replace(/[\uff10-\uff5e]/g, function (char) {
      return String.fromCharCode(char.charCodeAt(0) - 65248)
    })
    .replace(/\u3000/g, ' ') // 全角空格转换为半角空格
}

// 自定义排序函数
function getPriority(s: string): number {
  if (!s.length) return 2 // 空字符串视为其他符号
  const firstChar = s[0]
  if (/[a-zA-Z]/.test(firstChar)) return 0 // 字母
  if (/\d/.test(firstChar)) return 2 // 数字
  return 1 // 其他符号
}
function customSort(arr: string[]): string[] {
  return [...arr].sort((a, b) => {
    const priorityA = getPriority(a)
    const priorityB = getPriority(b)

    // 1. 按优先级排序（字母 > 数字 > 其他）
    if (priorityA !== priorityB) {
      return priorityA - priorityB
    }

    // 2. 同优先级时，进一步处理字母+数字的情况
    if (priorityA === 0) {
      // 字母开头
      // 提取字母部分和数字部分（如 "M10" → ["M", "10"]）
      const matchA = a.match(/^([a-zA-Z@#$%^&]*)(\d*)/)
      const matchB = b.match(/^([a-zA-Z@#$%^&]*)(\d*)/)

      const lettersA = matchA?.[1] || ''
      const lettersB = matchB?.[1] || ''
      const numA = matchA?.[2] ? parseInt(matchA[2], 10) : NaN
      const numB = matchB?.[2] ? parseInt(matchB[2], 10) : NaN

      // 2.1 先比较字母部分
      if (lettersA !== lettersB) {
        return lettersA.localeCompare(lettersB)
      }

      // 2.2 字母相同，比较数字部分
      if (!isNaN(numA) && !isNaN(numB)) {
        return numA - numB // 按数值排序
      }

      // 2.3 无法提取数字，按字典序排序
      return a.localeCompare(b)
    } else if (priorityA === 2) {
      // 数字
      // 2.2 字母相同，比较数字部分
      if (Number(a) && Number(b)) {
        return Number(a) - Number(b) // 按数值排序
      }
    }

    // 3. 其他情况（数字开头或其他符号开头），按字典序排序
    return a.localeCompare(b)
  })
}

export const splitCamelCase = (str: string) => {
  // 匹配两种情况：
  // 1. 小写字母或数字后跟大写字母（常规驼峰情况）
  // 2. 多个大写字母后跟一个大写字母和后续小写字母（如 "HPMin"）
  // return str.replace(/([a-z0-9])([A-Z])|([A-Z]+)([A-Z][a-z])/g, '$1$3 $2$4').split(' ');
  return str.replace(/([\p{L}]+)([A-Z][a-z])|([\p{L}\d]+)([A-Z])/gu, '$1$3 $2$4').split(' ')
  // \p{Ll}：匹配所有小写字母（包括非 ASCII 字母，如 α）。
  // \p{Nd}：匹配所有数字字符（包括非 ASCII 数字）。
  // [\p{Ll}\p{Nd}] +：匹配连续的小写字母或数字。
  /**
   * 第一部分：([\p{L}]+)([A-Z][a-z])
   * ([\p{L}]+)：这是第一个捕获组 $1，用于匹配非 ASCII 字符、小写字母或字母序列（如 εA）。
   * ([A-Z][a-z])：这是第二个捕获组 $2，用于匹配一个大写字母后跟一个小写字母（如 Min 中的 M 和 i）。
   * 第二部分：([\p{L}\d]+)([A-Z])
   * ([\p{L}\d]+)：这是第三个捕获组 $3，用于匹配非 ASCII 字符、小写字母或数字序列（如 ε1）。
   * ([A-Z])：这是第四个捕获组 $4，用于匹配单独的一个大写字母（如 A）。
   */
}
export const toCamelCase = (arr: string[]) => {
  return arr
    .map((word, index) => {
      // 对于第一个单词，全部小写；对于后续单词，首字母大写，其余小写
      if (index === 0) {
        return word.toLowerCase()
      }
      return word.charAt(0).toUpperCase() + word.slice(1).toLowerCase()
    })
    .join('')
}

export const parametersJons = (excelData: string) => {
  const lines = excelData.split('\n').filter((line) => line.trim())
  if (lines.length < 2) return {}

  // 解析表头
  const headers = lines[0].split('\t') // mon	公称	等级	条件
  if (headers.length < 4) throw new Error('Excel格式错误：必须包含[mon	公称	等级	条件]列')
  const monColumns = headers.filter((item, index) => index > 3) // 过滤出数据项
  const monColumnsIndex = monColumns.reduce((acc: any, arg: any, index: any) => {
    acc[arg] = index + 4
    return acc
  }, {})
  // 初始化结果容器
  const result: any = {}
  Object.keys(monColumnsIndex).forEach((key: any) => {
    result[key] = {}
    for (let i = 1; i < lines.length; i++) {
      const parts = lines[i]
        .replace(/ /g, '-')
        .replace(/(\t+)/g, (match) => {
          // 计算匹配到的制表符数量
          const count = match.length
          if (count > 1) {
            // 如果有两个或更多的制表符，则用 "\t-\t" 替换每个额外的制表符对
            // 注意：如果原始字符串中有奇数个连续的制表符，最后一个将保持不变
            return `\t${'-\t'.repeat(count - 1)}`
          } else {
            // 如果只有一个制表符，直接返回
            return match
          }
        })
        .split('\t')
        .filter(Boolean)
      const key0 = (parts[0] || '').trim().replace(/-/g, '')
      const key1 = (parts[1] || '').trim().replace(/-/g, '')
      let keyName = ''
      if (key1 === '') {
        keyName = key0
      } else {
        keyName = `${key0}${key1.charAt(0).toUpperCase() + key1.slice(1)}`
      }
      if (!result[key][keyName]) result[key][keyName] = {}
      const key2 = parts[2].trim().replace(/-/g, '')
      const key3 = parts[3].trim().replace(/-/g, '')
      // 获取对应的值
      const value = (parts[monColumnsIndex[key]] || '').trim().replace(/-/g, '')
      if (key2 !== '' && key3 !== '') {
        result[key][keyName]['condition'] = key3
        result[key][keyName][key2] = value
      } else if (key2 === '' && key3 !== '') {
        result[key][keyName][key3] = !!result[key][keyName][key3] ? result[key][keyName][key3] : value
      } else if (key2 !== '') {
        result[key][keyName][key2] = value
      } else {
        result[key][keyName] = value
      }
    }
  })
  // console.log(JSON.stringify(result))
  return result
}

export const lengthJson = (excelData: string) => {
  const lines = excelData.split('\n').filter((line) => line.trim())
  if (lines.length < 2) return {}

  // 解析表头
  const headers = lines[0].split('\t') // mon	公称	等级	条件
  if (headers.length < 1) throw new Error('Excel格式错误：必须包含[L]列')
  const monColumns = headers.filter((item, index) => index > 0) // 过滤出数据项
  const monColumnsIndex = monColumns.reduce((acc: any, arg: any, index: any) => {
    acc[arg] = index + 1
    return acc
  }, {})
  // 初始化结果容器
  const result: any = {}
  Object.keys(monColumnsIndex).forEach((key) => {
    for (let i = 1; i < lines.length; i++) {
      const parts = lines[i]
        .replace(/ /g, '-')
        .replace(/(\t+)/g, (match) => {
          // 计算匹配到的制表符数量
          const count = match.length
          if (count > 1) {
            // 如果有两个或更多的制表符，则用 "\t-\t" 替换每个额外的制表符对
            // 注意：如果原始字符串中有奇数个连续的制表符，最后一个将保持不变
            return `\t${'-\t'.repeat(count - 1)}`
          } else {
            // 如果只有一个制表符，直接返回
            return match
          }
        })
        .split('\t')
        .filter(Boolean)
      const key0 = (parts[0] || '').trim().replace(/-/g, '')
      const value = (parts[monColumnsIndex[key]] || '').trim().replace(/-/g, '')
      if (!!value) {
        if (!result[key]) result[key] = []
        result[key].push(key0)
      }
    }
  })
  // console.log(JSON.stringify(result))
  return result
}

export const drawingJson = (excelData: string) => {
  const lines = excelData.split('\n').filter((line) => line.trim())
  if (lines.length < 2) return {}

  // 解析表头
  const headers = lines[0].split('\t') // mon	公称	等级	条件
  if (headers.length < 1) throw new Error('Excel格式错误：必须包含[条件]列')
  const monColumns = headers.filter((item, index) => index > 0) // 过滤出数据项
  const monColumnsIndex = monColumns.reduce((acc: any, arg: any, index: any) => {
    acc[arg] = index + 1
    return acc
  }, {})
  // 初始化结果容器
  const result: any = {}
  Object.keys(monColumnsIndex).forEach((key) => {
    for (let i = 1; i < lines.length; i++) {
      const parts = lines[i]
        .replace(/ /g, '-')
        .replace(/(\t+)/g, (match) => {
          // 计算匹配到的制表符数量
          const count = match.length
          if (count > 1) {
            // 如果有两个或更多的制表符，则用 "\t-\t" 替换每个额外的制表符对
            // 注意：如果原始字符串中有奇数个连续的制表符，最后一个将保持不变
            return `\t${'-\t'.repeat(count - 1)}`
          } else {
            // 如果只有一个制表符，直接返回
            return match
          }
        })
        .split('\t')
        .filter(Boolean)
      const key0 = (parts[0] || '').trim().replace(/-/g, '')
      const value = (parts[monColumnsIndex[key]] || '').trim().replace(/-/g, '')
      if (!!value) {
        if (!result[key]) result[key] = {}
        result[key][key0.toUpperCase()] = value
      }
    }
  })
  // console.log(JSON.stringify(result))
  return result
}

export const parametersToTable = (tableData: any, editForm: any) => {
  const result = {} as any
  const i = {} as any
  for (const [key, value] of Object.entries<{ d?: any }>(editForm.parameters || {})) {
    if (value.d) {
      i[value.d] = (i[value.d] || 0) + 1
      if (result[value.d]) {
        result[value.d + i[value.d]] = key
      } else {
        result[value.d] = key
      }
    } else {
      result[key] = key
    }
  }
  const keysSort = customSort(Object.keys(result || {}))
  tableData.column = Object.keys(editForm.parameters || {})
  // console.log('editForm.parameters:', JSON.stringify(editForm.parameters))
  tableData.list = []
  // {"M6":{"d":"6","P":"1|1.25","bMin":{"L≤125":"18","125＜L≤200":"24","L＞200":"37"},...
  const rows = editForm.parameters[tableData.column[0]] || {}
  Object.keys(rows).forEach((row) => {
    const mon = typeof row === 'string' ? splitCamelCase(row) : [row, false]
    const data: any = {
      mon: mon[0] ? mon[0] : row,
      nominal: mon[1] ? mon[1] : '',
      leve: '',
      condition: '',
      values: {},
    }
    // "bMin":{"L≤125":"18","125＜L≤200":"24","L＞200":"37"}
    // "eMin":{"condition":"(d≤16&-L≤150)-|-L≤10-*-d","A":"11.05","B":"10.89"}
    // console.log('rows[row]:', typeof rows[row], rows.hasOwnProperty(row), rows[row])
    if (rows.hasOwnProperty(row) && typeof rows[row] === 'object' && rows[row] !== null && !Array.isArray(rows[row])) {
      if (rows[row]['condition']) {
        // "eMin":{"condition":"(d≤16&-L≤150)-|-L≤10-*-d","A":"11.05","B":"10.89"}
        if (rows[row]['A']) {
          const newDataA = { ...data, leve: 'A', condition: rows[row]['condition'] }
          const haveListA = tableData.list.some((l: any) => {
            return (
              l.mon === newDataA.mon && l.nominal === newDataA.nominal && l.leve === newDataA.leve && l.condition === newDataA.condition
            )
          })
          if (!haveListA) {
            tableData.column.forEach((col: string) => {
              const colValue = editForm.parameters[col][row]
              newDataA.values[col] = typeof colValue === 'object' ? colValue['A'] || '' : colValue || ''
            })
            tableData.list.push(JSON.parse(JSON.stringify(newDataA)))
          }
        }
        if (rows[row]['B']) {
          const newDataB = { ...data, leve: 'B' }
          const haveListB = tableData.list.some((l: any) => {
            return (
              l.mon === newDataB.mon && l.nominal === newDataB.nominal && l.leve === newDataB.leve && l.condition === newDataB.condition
            )
          })
          if (!haveListB) {
            tableData.column.forEach((col: string) => {
              const colValue = editForm.parameters[col][row]
              newDataB.values[col] = typeof colValue === 'object' ? colValue['B'] || '' : colValue || ''
            })
            tableData.list.push(JSON.parse(JSON.stringify(newDataB)))
          }
        }
      } else {
        // "bMin": { "L≤125": "18", "125＜L≤200": "24", "L＞200": "37" }
        Object.keys(rows[row]).forEach((key) => {
          const newDataC = { ...data, condition: key }
          const haveListC = tableData.list.some((l: any) => {
            return (
              l.mon === newDataC.mon && l.nominal === newDataC.nominal && l.leve === newDataC.leve && l.condition === newDataC.condition
            )
          })
          if (!haveListC) {
            tableData.column.forEach((col: string) => {
              const colValue = editForm.parameters[col][row]
              newDataC.values[col] = typeof colValue === 'object' ? colValue[key] || '' : colValue || ''
            })
            tableData.list.push(JSON.parse(JSON.stringify(newDataC)))
          }
        })
      }
    } else {
      const newData = { ...data, condition: '' }
      const haveList = tableData.list.some((l: any) => {
        return l.mon === newData.mon && l.nominal === newData.nominal && l.leve === newData.leve && l.condition === newData.condition
      })
      if (!haveList) {
        tableData.column.forEach((col: string) => {
          const colValue = editForm.parameters[col][row]
          newData.values[col] = colValue || ''
        })
        tableData.list.push(JSON.parse(JSON.stringify(newData)))
      }
    }
  })
  return tableData
}

export const lengthToTable = (tableData: any, editForm: any) => {
  // console.log(JSON.stringify(editForm.length))
  // {"M6":["1"],"M8":["2"],"M10":["2"],"M14":["6"],"M16":["6"],"M20":["12"],"M22":["27.5"],"M27":["27.5"],"M36":["27.5"],"#6":["6"],"#8":["12"]}
  tableData.column = Object.keys(editForm.length || {})
  const lArr = Array.from(
    new Set(
      // 使用 Set 去重
      Object.values(editForm.length) // 获取对象的所有值（数组）
        .flat() // 将二维数组展平为一维数组
        .map(Number) // 将字符串转换为数字
    )
  ).sort((a, b) => a - b) // 按升序排序（可选）
  tableData.list = []
  lArr.forEach((l) => {
    const data: any = {
      L: l,
      values: {},
    }
    tableData.column.forEach((col: string) => {
      data.values[col] = editForm.length[col]
    })
    tableData.list.push(JSON.parse(JSON.stringify(data)))
  })
}

export const drawingLimitTable = (tableData: any, editForm: any) => {
  // console.log(JSON.stringify(editForm.length))
  // {"M6":{"3d":"L<30","cad":"L<30","svg":"L<30"},"M8":{"3d":"d<40","cad":"d<40","svg":"d<40"}}
  // console.log('editForm.drawing_limit:', tableData.column.length)
  tableData.column = Object.keys(editForm.drawing_limit || {})
  tableData.list = []
  const line = ['SVG', 'CAD', '3D']
  if (tableData.column.length > 0) {
    line.forEach((item: string) => {
      const data: any = {
        condition: item,
        values: editForm.drawing_limit[item],
      }
      tableData.list.push(JSON.parse(JSON.stringify(data)))
    })
  }
}
