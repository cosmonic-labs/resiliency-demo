(function(){const r=document.createElement("link").relList;if(r&&r.supports&&r.supports("modulepreload"))return;for(const t of document.querySelectorAll('link[rel="modulepreload"]'))n(t);new MutationObserver(t=>{for(const o of t)if(o.type==="childList")for(const s of o.addedNodes)s.tagName==="LINK"&&s.rel==="modulepreload"&&n(s)}).observe(document,{childList:!0,subtree:!0});function a(t){const o={};return t.integrity&&(o.integrity=t.integrity),t.referrerPolicy&&(o.referrerPolicy=t.referrerPolicy),t.crossOrigin==="use-credentials"?o.credentials="include":t.crossOrigin==="anonymous"?o.credentials="omit":o.credentials="same-origin",o}function n(t){if(t.ep)return;t.ep=!0;const o=a(t);fetch(t.href,o)}})();const d=3e3;function g(e){e.classList.remove("animate-exit"),e.classList.add("animate-enter")}function m(e){e.classList.add("animate-exit"),e.classList.remove("animate-enter")}function u(e){g(e),setTimeout(()=>m(e),d)}function f(e){const r=document.querySelectorAll(e);r.forEach((a,n)=>{setTimeout(()=>{u(a),setInterval(()=>u(a),r.length*d)},n*d)})}const h={ams:{os:{"Mac OSX":15},browsers:{Chrome:15},visits:15},iad:{os:{Android:3,"Mac OSX":118,UNKNOWN:1,"Windows Phone OS":1,"Windows Vista":1,"Xbox One":1,iPhone:2},browsers:{Chrome:104,Edge:2,Firefox:14,"Internet Explorer":2,Opera:2,Safari:1,"Xbox One":1,"misc crawler":1},visits:127},ord:{os:{Android:3,"Mac OSX":80,UNKNOWN:1,"Windows Vista":1,"Xbox One":1,iPhone:2},browsers:{Chrome:78,Edge:2,Firefox:2,"Internet Explorer":2,Opera:2,Safari:1,"Xbox One":1,"misc crawler":1},visits:89}},p={ams:"Amsterdam, Netherlands",arn:"Stockholm, Sweden",atl:"Atlanta, Georgia (US)",bog:"Bogotá, Colombia",bom:"Mumbai, India",bos:"Boston, Massachusetts (US)",cdg:"Paris, France",den:"Denver, Colorado (US)",dfw:"Dallas, Texas (US)",ewr:"Secaucus, NJ (US)",eze:"Ezeiza, Argentina",fra:"Frankfurt, Germany",gdl:"Guadalajara, Mexico",gig:"Rio de Janeiro, Brazil",gru:"Sao Paulo, Brazil",hkg:"Hong Kong, Hong Kong",iad:"Ashburn, Virginia (US)",jnb:"Johannesburg, South Africa",lax:"Los Angeles, California (US)",lhr:"London, United Kingdom",mad:"Madrid, Spain",mia:"Miami, Florida (US)",nrt:"Tokyo, Japan",ord:"Chicago, Illinois (US)",otp:"Bucharest, Romania",phx:"Phoenix, Arizona (US)",qro:"Querétaro, Mexico",scl:"Santiago, Chile",sea:"Seattle, Washington (US)",sin:"Singapore, Singapore",sjc:"San Jose, California (US)",syd:"Sydney, Australia",waw:"Warsaw, Poland",yul:"Montreal, Canada",yyz:"Toronto, Canada"};function b(e){const r=new Set,a=new Set;for(const s in e){const i=e[s];for(const c in i.os)r.add(c);for(const c in i.browsers)a.add(c)}const n=[...r].sort(),t=[...a].sort(),o=[];for(const s in e){const i=e[s],c=[s,i.visits,...n.map(l=>i.os[l]??0),...t.map(l=>i.browsers[l]??0)];o.push(c)}return{osHeaders:n,browserHeaders:t,regions:o}}function S(e){const r=e.osHeaders,a=e.browserHeaders,{regions:n}=e,t=r.length+a.length+1;return`
  <table class="[&_th]:p-2 [&_th]:align-top [&_td]:p-2 bg-slate-500 rounded overflow-hidden">
    <thead class="text-sm whitespace-nowrap text-white">
      <tr>
        <th colspan="2" rowspan="2" class="bg-sky-950/10">Visits</th>
        <th colspan="${n.length}">Region</th>
      </tr>
      <tr>
        ${n.map(o=>`<th scope="col" class="bg-sky-950/30">${p[o[0]]||o[0]}</th>`).join("")}
      </tr>
    </thead>
    <tbody class="text-black [&_th]:text-white text-left">
      ${Array.from({length:t}).map((o,s)=>s===0?`<tr>
            <th class="bg-sky-950/30" colspan="2" scope="row">Total</th>
            ${n.map(i=>`<td class="bg-slate-300/80">${i[1]}</td>`).join("")}
          </tr>`:s<=r.length?`<tr>
            ${s===1?`<th rowspan="${r.length}" class="bg-sky-950/20 text-center [writing-mode:vertical-lr]">OS</th>`:""}
            <th scope="row">${r[s-1]}</th>
            ${n.map(i=>`<td class="bg-slate-300">${i[s+1]}</td>`).join("")}
          </tr>`:`<tr>
          ${s===r.length+1?`<th rowspan="${a.length}" class="bg-sky-950/30 text-center [writing-mode:vertical-lr]">Browser</th>`:""}
          <th class="bg-sky-950/10" scope="row">${a[s-r.length-1]}</th>
          ${n.map(i=>`<td class="bg-slate-300/80">${i[s+1]}</td>`).join("")}
        </tr>`).join("")}
    </tbody>
</table>`}function w(e){try{return e?JSON.parse(e):h}catch{return h}}function y(e,r){var n;const a=document.querySelector(r);if(a){const t=((n=document.querySelector(e))==null?void 0:n.textContent)??"",o=b(w(t));a.innerHTML=S(o)}}function x(){f(".js-cycle"),y("#data",'[data-js="data-table"]')}x();
