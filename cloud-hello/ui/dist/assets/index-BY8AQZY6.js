(function(){const o=document.createElement("link").relList;if(o&&o.supports&&o.supports("modulepreload"))return;for(const t of document.querySelectorAll('link[rel="modulepreload"]'))s(t);new MutationObserver(t=>{for(const r of t)if(r.type==="childList")for(const n of r.addedNodes)n.tagName==="LINK"&&n.rel==="modulepreload"&&s(n)}).observe(document,{childList:!0,subtree:!0});function a(t){const r={};return t.integrity&&(r.integrity=t.integrity),t.referrerPolicy&&(r.referrerPolicy=t.referrerPolicy),t.crossOrigin==="use-credentials"?r.credentials="include":t.crossOrigin==="anonymous"?r.credentials="omit":r.credentials="same-origin",r}function s(t){if(t.ep)return;t.ep=!0;const r=a(t);fetch(t.href,r)}})();const d=3e3;function g(e){e.classList.remove("animate-exit"),e.classList.add("animate-enter")}function m(e){e.classList.add("animate-exit"),e.classList.remove("animate-enter")}function u(e){g(e),setTimeout(()=>m(e),d)}function f(e){const o=document.querySelectorAll(e);o.forEach((a,s)=>{setTimeout(()=>{u(a),setInterval(()=>u(a),o.length*d)},s*d)})}const h={ams:{os:{"Mac OSX":15},browsers:{Chrome:15},visits:15},iad:{os:{Android:3,"Mac OSX":118,UNKNOWN:1,"Windows Phone OS":1,"Windows Vista":1,"Xbox One":1,iPhone:2},browsers:{Chrome:104,Edge:2,Firefox:14,"Internet Explorer":2,Opera:2,Safari:1,"Xbox One":1,"misc crawler":1},visits:127},ord:{os:{Android:3,"Mac OSX":80,UNKNOWN:1,"Windows Vista":1,"Xbox One":1,iPhone:2},browsers:{Chrome:78,Edge:2,Firefox:2,"Internet Explorer":2,Opera:2,Safari:1,"Xbox One":1,"misc crawler":1},visits:89}},p={ams:"Amsterdam, Netherlands",arn:"Stockholm, Sweden",atl:"Atlanta, Georgia (US)",bog:"Bogotá, Colombia",bom:"Mumbai, India",bos:"Boston, Massachusetts (US)",cdg:"Paris, France",den:"Denver, Colorado (US)",dfw:"Dallas, Texas (US)",ewr:"Secaucus, NJ (US)",eze:"Ezeiza, Argentina",fra:"Frankfurt, Germany",gdl:"Guadalajara, Mexico",gig:"Rio de Janeiro, Brazil",gru:"Sao Paulo, Brazil",hkg:"Hong Kong, Hong Kong",iad:"Ashburn, Virginia (US)",jnb:"Johannesburg, South Africa",lax:"Los Angeles, California (US)",lhr:"London, United Kingdom",mad:"Madrid, Spain",mia:"Miami, Florida (US)",nrt:"Tokyo, Japan",ord:"Chicago, Illinois (US)",otp:"Bucharest, Romania",phx:"Phoenix, Arizona (US)",qro:"Querétaro, Mexico",scl:"Santiago, Chile",sea:"Seattle, Washington (US)",sin:"Singapore, Singapore",sjc:"San Jose, California (US)",syd:"Sydney, Australia",waw:"Warsaw, Poland",yul:"Montreal, Canada",yyz:"Toronto, Canada"};function b(e){const o=new Set,a=new Set;for(const n in e){const i=e[n];for(const c in i.os)o.add(c);for(const c in i.browsers)a.add(c)}const s=[...o].sort(),t=[...a].sort(),r=[];for(const n in e){const i=e[n],c=[n,i.visits,...s.map(l=>i.os[l]??0),...t.map(l=>i.browsers[l]??0)];r.push(c)}return{osHeaders:s,browserHeaders:t,regions:r}}function S(e){const o=e.osHeaders,a=e.browserHeaders,{regions:s}=e,t=o.length+a.length+1;return`
  <table class="[&_th]:p-2 [&_th]:align-top [&_td]:p-2 bg-slate-500 rounded overflow-hidden">
    <thead class="text-sm whitespace-nowrap text-white">
      <tr>
        <th colspan="2" rowspan="2" class="bg-sky-950/10">Visits</th>
        <th colspan="${s.length}">Region</th>
      </tr>
      <tr>
        ${s.map(r=>`<th scope="col" class="bg-sky-950/30">${p[r[0]]}</th>`).join("")}
      </tr>
    </thead>
    <tbody class="text-black [&_th]:text-white text-left">
      ${Array.from({length:t}).map((r,n)=>n===0?`<tr>
            <th class="bg-sky-950/30" colspan="2" scope="row">Total</th>
            ${s.map(i=>`<td class="bg-slate-300/80">${i[1]}</td>`).join("")}
          </tr>`:n<=o.length?`<tr>
            ${n===1?`<th rowspan="${o.length}" class="bg-sky-950/20 text-center [writing-mode:vertical-lr]">OS</th>`:""}
            <th scope="row">${o[n-1]}</th>
            ${s.map(i=>`<td class="bg-slate-300">${i[n+1]}</td>`).join("")}
          </tr>`:`<tr>
          ${n===o.length+1?`<th rowspan="${a.length}" class="bg-sky-950/30 text-center [writing-mode:vertical-lr]">Browser</th>`:""}
          <th class="bg-sky-950/10" scope="row">${a[n-o.length-1]}</th>
          ${s.map(i=>`<td class="bg-slate-300/80">${i[n+1]}</td>`).join("")}
        </tr>`).join("")}
    </tbody>
</table>`}function w(e){try{return e?JSON.parse(e):h}catch{return h}}function y(e,o){var s;const a=document.querySelector(o);if(a){const t=((s=document.querySelector(e))==null?void 0:s.textContent)??"",r=b(w(t));a.innerHTML=S(r)}}function x(){f(".js-cycle"),y("#data",'[data-js="data-table"]')}x();
