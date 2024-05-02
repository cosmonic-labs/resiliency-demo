(function(){const r=document.createElement("link").relList;if(r&&r.supports&&r.supports("modulepreload"))return;for(const e of document.querySelectorAll('link[rel="modulepreload"]'))o(e);new MutationObserver(e=>{for(const s of e)if(s.type==="childList")for(const c of s.addedNodes)c.tagName==="LINK"&&c.rel==="modulepreload"&&o(c)}).observe(document,{childList:!0,subtree:!0});function n(e){const s={};return e.integrity&&(s.integrity=e.integrity),e.referrerPolicy&&(s.referrerPolicy=e.referrerPolicy),e.crossOrigin==="use-credentials"?s.credentials="include":e.crossOrigin==="anonymous"?s.credentials="omit":s.credentials="same-origin",s}function o(e){if(e.ep)return;e.ep=!0;const s=n(e);fetch(e.href,s)}})();const u=3e3;function p(t){t.classList.remove("animate-exit"),t.classList.add("animate-enter")}function b(t){t.classList.add("animate-exit"),t.classList.remove("animate-enter")}function d(t){p(t),setTimeout(()=>b(t),u)}function m(t){const r=document.querySelectorAll(t);r.forEach((n,o)=>{setTimeout(()=>{d(n),setInterval(()=>d(n),r.length*u)},o*u)})}const f={den:{os:{"Mac OSX":29,UNKNOWN:1},browsers:{Chrome:8,Firefox:21,"HTTP Library":1},visits:114},ord:{os:{Banana:29,UNKNOWN:1},browsers:{"HTTP Library":1,Safari:8,Firefox:21},visits:114},atl:{os:{Linux:1},browsers:{Opera:8},visits:114}};function h(t){const r=new Set,n=new Set;for(const c in t){const a=t[c];for(const i in a.os)r.add(i);for(const i in a.browsers)n.add(i)}const o=[...r].sort(),e=[...n].sort(),s=[];for(const c in t){const a=t[c],i=[c,a.visits,...o.map(l=>a.os[l]??0),...e.map(l=>a.browsers[l]??0)];s.push(i)}return{osHeaders:o,browserHeaders:e,rows:s}}function g(t){const r=t.osHeaders,n=t.browserHeaders;return`
  <table class="[&_th]:p-2 [&_th]:align-top [&_td]:p-2 bg-slate-300 rounded overflow-hidden">
    <thead class="bg-slate-500 text-sm whitespace-nowrap">
      <tr >
        <th class="bg-black/10" rowspan="2" scope="col">Region</th>
        <th class="bg-black/20" rowspan="2" scope="col">Visits</th>
        <th class="bg-black/30" colspan="${r.length}" scope="colgroup">OS</th>
        <th class="bg-black/40" colspan="${n.length}" scope="colgroup">Browser</th>
      </tr>
      <tr>
        ${r.map(o=>`<th scope="col" class="bg-black/40">${o}</th>`).join("")}
        ${n.map(o=>`<th scope="col" class="bg-black/30">${o}</th>`).join("")}
      </tr>
    </thead>
    <tbody class="text-black">
      ${t.rows.map(o=>`
        <tr>
          ${o.map(e=>`<td>${e}</td>`).join("")}
        </tr>
      `).join("")}
    </tbody>
</table>`}function w(t){try{return t?JSON.parse(t):f}catch{return f}}function y(t,r){var o;const n=document.querySelector(r);if(n){const e=((o=document.querySelector(t))==null?void 0:o.textContent)??"",s=h(w(e));n.innerHTML=g(s)}}function L(){m(".js-cycle"),y("#data",'[data-js="data-table"]')}L();
