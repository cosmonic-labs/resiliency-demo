import example from "./data/example";



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

  const rows: (string | number)[][] = []

  for (const region in data) {
    const rowData = data[region]

    const row = [
      region,
      rowData.visits,
      ...osHeaders.map(os => rowData.os[os] ?? 0),
      ...browserHeaders.map(browser => rowData.browsers[browser] ?? 0)
    ]

    rows.push(row)
  }

  return {osHeaders, browserHeaders, rows}
}

function getTableHtml(data: ReturnType<typeof parseData>) {
  // I really probably should have just used react for this project...
  const osth = data.osHeaders
  const brth = data.browserHeaders
  return `
  <table class="[&_th]:p-2 [&_th]:align-top [&_td]:p-2 bg-slate-300 rounded overflow-hidden">
    <thead class="bg-slate-500 text-sm whitespace-nowrap">
      <tr >
        <th class="bg-black/10" rowspan="2" scope="col">Region</th>
        <th class="bg-black/20" rowspan="2" scope="col">Visits</th>
        <th class="bg-black/30" colspan="${osth.length}" scope="colgroup">OS</th>
        <th class="bg-black/40" colspan="${brth.length}" scope="colgroup">Browser</th>
      </tr>
      <tr>
        ${osth.map(os => `<th scope="col" class="bg-black/40">${os}</th>`).join('')}
        ${brth.map(browser => `<th scope="col" class="bg-black/30">${browser}</th>`).join('')}
      </tr>
    </thead>
    <tbody class="text-black">
      ${data.rows.map(row => `
        <tr>
          ${row.map(cell => `<td>${cell}</td>`).join('')}
        </tr>
      `).join('')}
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