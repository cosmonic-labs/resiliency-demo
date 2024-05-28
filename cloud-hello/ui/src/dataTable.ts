import example from "./data/example";
import codes from "./data/codes";


type RegionDataRow = {
  os: Record<string, number>;
  browsers: Record<string, number>;
  visits: number;
}

type RegionData = {
  [key: string]: RegionDataRow;
}

function parseData(data: RegionData) {
  const osList = new Set<string>()
  const browserList = new Set<string>()

  for (const region in data) {
    const row = data[region]

    for (const os in row.os) {
      osList.add(os)
    }
    for (const browser in row.browsers) {
      browserList.add(browser)
    }
  }

  const osHeaders = [...osList].sort()
  const browserHeaders = [...browserList].sort()

  const regions: (string | number)[][] = []

  for (const code in data) {
    const regionData = data[code]

    const region = [
      code,
      regionData.visits,
      ...osHeaders.map(os => regionData.os[os] ?? 0),
      ...browserHeaders.map(browser => regionData.browsers[browser] ?? 0)
    ]

    regions.push(region)
  }

  return {osHeaders, browserHeaders, regions}
}

function getTableHtml(data: ReturnType<typeof parseData>) {
  // I really probably should have just used react for this project...
  const osth = data.osHeaders
  const brth = data.browserHeaders
  const {regions} = data
  const totalVals = osth.length + brth.length + 1
  return `
  <table class="[&_th]:p-2 [&_th]:align-top [&_td]:p-2 bg-slate-500 rounded overflow-hidden">
    <thead class="text-sm whitespace-nowrap text-white">
      <tr>
        <th colspan="2" rowspan="2" class="bg-sky-950/10">Visits</th>
        <th colspan="${regions.length}">Region</th>
      </tr>
      <tr>
        ${regions.map(region => `<th scope="col" class="bg-sky-950/30">${codes[region[0]] || region[0]}</th>`).join('')}
      </tr>
    </thead>
    <tbody class="text-black [&_th]:text-white text-left">
      ${Array.from({length: totalVals}).map((_, i) => {
        if (i === 0) {
          return `<tr>
            <th class="bg-sky-950/30" colspan="2" scope="row">Total</th>
            ${regions.map(region => `<td class="bg-slate-300/80">${region[1]}</td>`).join('')}
          </tr>`
        }
        if (i <= osth.length) {
          return `<tr>
            ${i === 1 ? `<th rowspan="${osth.length}" class="bg-sky-950/20 text-center [writing-mode:vertical-lr]">OS</th>` : ''}
            <th scope="row">${osth[i - 1]}</th>
            ${regions.map(region => `<td class="bg-slate-300">${region[i + 1]}</td>`).join('')}
          </tr>`
        }
        return `<tr>
          ${i === osth.length + 1 ? `<th rowspan="${brth.length}" class="bg-sky-950/30 text-center [writing-mode:vertical-lr]">Browser</th>` : ''}
          <th class="bg-sky-950/10" scope="row">${brth[i - osth.length - 1]}</th>
          ${regions.map(region => `<td class="bg-slate-300/80">${region[i + 1]}</td>`).join('')}
        </tr>`
      }).join('')}
    </tbody>
</table>`
}

function getDataOrFallback(string: string): RegionData {
  try {
    return string ? (JSON.parse(string)) : example
  } catch (e) {
    return example
  }
}

function dataTable(dataSelector: string, tableSelector: string) {
  const tableContainer = document.querySelector(tableSelector)
  if (tableContainer) {
    const dataScriptText = document.querySelector(dataSelector)?.textContent ?? ''
    const data = parseData(getDataOrFallback(dataScriptText))
    tableContainer.innerHTML = getTableHtml(data)
  }
}

export {dataTable}
