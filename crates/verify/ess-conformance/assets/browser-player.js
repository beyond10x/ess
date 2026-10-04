// This module renders Rust display values and moves original bytes; it never interprets a model.
export const MAX_FRAME = 64 * 1024 * 1024;
const MAX_DISPLAY_DEPTH=1024;
// An Object display layer is an object, an entries array and a key/value pair array.
// The document/scenarios/card envelope and terminal display object add four levels.
const MAX_CONTROL_DEPTH=3*MAX_DISPLAY_DEPTH+4;
const encoder = new TextEncoder(), decoder = new TextDecoder('utf-8', {fatal:true});
export const text = bytes => decoder.decode(bytes);
export function failure(code) { return new Error(code); }
export function pathValid(path) {
  return typeof path === 'string' && encoder.encode(path).length <= 1024 && path.length > 0 &&
    !/[\\:%?#\p{Cc}]/u.test(path) && path.split('/').every(x => x && x !== '.' && x !== '..');
}
function closed(value, fields) {
  if (!value || Array.isArray(value) || typeof value !== 'object' ||
      Object.keys(value).length !== fields.length || fields.some(k => !Object.hasOwn(value,k))) throw failure('invalid_bundle');
}
export function parseControl(bytes) {
  // Only manifest/presentation control JSON reaches this function. Reject duplicate member
  // spellings before JSON.parse can erase them; source and execution bytes never enter here.
  const original=text(bytes), stack=[];let at=0;
  while(at<original.length){const ch=original[at];
    if(ch==='"'){
      const begin=at++;let escaped=false;
      while(at<original.length){const c=original[at++];if(escaped){escaped=false;continue;}if(c==='\\'){escaped=true;continue;}if(c==='"')break;}
      let following=at;while(/\s/u.test(original[following]??'')&&following<original.length)following++;
      if(original[following]===':'){
        const keys=stack.at(-1);if(!(keys instanceof Set))throw failure('invalid_bundle');
        const key=JSON.parse(original.slice(begin,at));if(keys.has(key))throw failure('invalid_bundle');keys.add(key);
      }
    }else{at++;if(ch==='{')stack.push(new Set());else if(ch==='[')stack.push(null);else if(ch==='}'||ch===']')stack.pop();}
    if(stack.length>MAX_CONTROL_DEPTH)throw failure('resource_limit');
  }
  try{return JSON.parse(original);}catch(_){throw failure('invalid_bundle');}
}
export async function fetchBytes(path, limit, total = {bytes:0}) {
  const response = await fetch(path, {cache:'no-store'});
  if (!response.ok) throw failure(path === 'runner.wasm' ? 'build_required' : 'invalid_bundle');
  const reader = response.body.getReader(), chunks = []; let size = 0;
  try {
    while (true) {
      const {done,value} = await reader.read(); if (done) break;
      size += value.length; total.bytes += value.length;
      if (size > limit || total.bytes > MAX_FRAME) { await reader.cancel(); throw failure('resource_limit'); }
      chunks.push(value);
    }
  } finally { reader.releaseLock(); }
  const out = new Uint8Array(size); let offset = 0;
  for (const chunk of chunks) { out.set(chunk,offset); offset += chunk.length; }
  return out;
}
export async function hash(bytes) {
  return [...new Uint8Array(await crypto.subtle.digest('SHA-256',bytes))].map(n => n.toString(16).padStart(2,'0')).join('');
}
export async function acquire() {
  const total = {bytes:0};
  const original = await fetchBytes('browser.json',256*1024,total);
  const manifest = parseControl(original); // control metadata only, never source/suite/input.
  closed(manifest,['format','abi','sources','execution','presentation','generator']);
  if (manifest.format !== 'ess-conformance-browser/1' || manifest.abi !== 'ess-conformance-browser-abi/1') throw failure('incompatible_abi');
  closed(manifest.generator,['package','version','semantic_revision']);
  if (manifest.generator.package !== 'ess-conformance' || manifest.generator.semantic_revision !== 1 || typeof manifest.generator.version !== 'string') throw failure('invalid_bundle');
  closed(manifest.execution,['kind','file']);
  if (!['ordinary_suite','coverage_input'].includes(manifest.execution.kind) || !Array.isArray(manifest.sources) || manifest.sources.length === 0 || manifest.sources.length > 1024) throw failure('invalid_bundle');
  const refs = [...manifest.sources,manifest.execution.file,manifest.presentation], seen = new Set(), blobs = [];
  for (const ref of refs) {
    closed(ref,['path','sha256','byte_length']);
    if (!pathValid(ref.path) || seen.has(ref.path) || !Number.isSafeInteger(ref.byte_length) || ref.byte_length < 0 || ref.byte_length > MAX_FRAME || !/^[a-f0-9]{64}$/u.test(ref.sha256)) throw failure('invalid_bundle');
    seen.add(ref.path);
  }
  for (const ref of refs) {
    const bytes = await fetchBytes(ref.path,ref.byte_length,total);
    if (bytes.length !== ref.byte_length || await hash(bytes) !== ref.sha256) throw failure('invalid_bundle');
    blobs.push({path:ref.path,bytes});
  }
  const displayBytes = blobs.at(-1).bytes;
  if (displayBytes.length > 32*1024*1024) throw failure('resource_limit');
  const display = parseControl(displayBytes);
  validatePresentation(display);
  return {original,manifest,blobs,display};
}
function validateValue(value, budget, depth = 0) {
  if (++budget.nodes > 1000000 || depth > MAX_DISPLAY_DEPTH) throw failure('resource_limit');
  if (value?.kind === 'null') { closed(value,['kind']); return; }
  closed(value,['kind','value']);
  switch (value.kind) {
    case 'bool': if (typeof value.value !== 'boolean') throw failure('invalid_bundle'); break;
    case 'text': if (typeof value.value !== 'string') throw failure('invalid_bundle'); break;
    case 'number_text': if (typeof value.value !== 'string' || !/^-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?$/u.test(value.value)) throw failure('invalid_bundle'); break;
    case 'list': if (!Array.isArray(value.value)) throw failure('invalid_bundle'); for (const v of value.value) validateValue(v,budget,depth+1); break;
    case 'object': {
      if (!Array.isArray(value.value)) throw failure('invalid_bundle'); const seen = new Set();
      for (const pair of value.value) {
        if (!Array.isArray(pair) || pair.length !== 2 || typeof pair[0] !== 'string' || seen.has(pair[0])) throw failure('invalid_bundle');
        seen.add(pair[0]); validateValue(pair[1],budget,depth+1);
      } break;
    }
    default: throw failure('invalid_bundle');
  }
}
export function validatePresentation(display) {
  closed(display,['format','selected_digest','parent_digests','sources','model','suite','scenarios']);
  if (display.format !== 'ess-conformance-browser-presentation/1' || !Array.isArray(display.scenarios) || !Array.isArray(display.parent_digests) || !Array.isArray(display.sources)) throw failure('invalid_bundle');
  const digest=value=>typeof value==='string'&&/^sha256:[a-f0-9]{64}$/u.test(value);
  if(!digest(display.selected_digest)||display.parent_digests.some(value=>!digest(value))||display.sources.length===0||display.sources.length>1024)throw failure('invalid_bundle');
  const sourceLabels=new Set();for(const source of display.sources){closed(source,['path','sha256','byte_length']);if(!pathValid(source.path)||sourceLabels.has(source.path)||typeof source.sha256!=='string'||!/^[a-f0-9]{64}$/u.test(source.sha256)||!Number.isSafeInteger(source.byte_length)||source.byte_length<0||source.byte_length>MAX_FRAME)throw failure('invalid_bundle');sourceLabels.add(source.path);}
  const budget = {nodes:0}; validateValue(display.model,budget); validateValue(display.suite,budget);
  const seen = new Set();
  for (const card of display.scenarios) { closed(card,['id','declaration']); if (typeof card.id !== 'string' || card.id.length===0 || encoder.encode(card.id).length>4096 || seen.has(card.id)) throw failure('invalid_bundle'); seen.add(card.id); validateValue(card.declaration,budget); }
}
export class Writer {
  constructor() { this.chunks=[]; this.length=0; }
  raw(bytes) { this.length += bytes.length; if (this.length > MAX_FRAME) throw failure('resource_limit'); this.chunks.push(bytes); }
  word(n) { if (!Number.isInteger(n) || n < 0 || n > 0xffffffff) throw failure('invalid_frame'); const b=new Uint8Array(4); new DataView(b.buffer).setUint32(0,n,true); this.raw(b); }
  bytes(bytes) { this.word(bytes.length); this.raw(bytes); }
  string(value) { this.bytes(encoder.encode(value)); }
  finish() { const out=new Uint8Array(this.length); let at=0; for(const b of this.chunks){out.set(b,at);at+=b.length;} return out; }
}
export function request(opcode,id,payload) {
  const writer=new Writer(); writer.raw(encoder.encode('ESBW')); [1,0,opcode,id,payload.length].forEach(n=>writer.word(n)); writer.raw(payload); return writer.finish();
}
export class Reader {
  constructor(bytes){ this.bytes=bytes; this.at=0; }
  raw(length){ if (!Number.isInteger(length) || length < 0 || this.at+length>this.bytes.length) throw failure('invalid_frame'); const out=this.bytes.slice(this.at,this.at+length); this.at+=length;return out; }
  word(){const b=this.raw(4);return new DataView(b.buffer,b.byteOffset,4).getUint32(0,true);}
  blob(){return this.raw(this.word());}
  string(){return text(this.blob());}
  end(){if(this.at!==this.bytes.length)throw failure('invalid_frame');}
}
const DISPLAY_PAGE=400;
function renderValue(value, offset=0) {
  // At most four bounded display panels and 200 scenario options are mounted at once.
  // Paths are display navigation only; they never become model/execution authority.
  const stack=[{value,path:'$'}], lines=[];let visited=0;
  while(stack.length && lines.length<DISPLAY_PAGE){
    const {value:v,path}=stack.pop();let scalar;
    switch(v.kind){
      case 'null':scalar='null';break;case 'bool':scalar=String(v.value);break;
      case 'text':scalar=JSON.stringify(v.value);break;case 'number_text':scalar=v.value;break;
      case 'list':scalar=`[${v.value.length} entries]`;for(let i=v.value.length-1;i>=0;i--)stack.push({value:v.value[i],path:`${path}[${i}]`});break;
      case 'object':scalar=`{${v.value.length} fields}`;for(let i=v.value.length-1;i>=0;i--){const [k,x]=v.value[i];stack.push({value:x,path:`${path}[${JSON.stringify(k)}]`});}break;
      default:throw failure('invalid_bundle');
    }
    if(visited++>=offset)lines.push(`${path}: ${scalar}`);
  }
  return {text:lines.join('\n'),omitted:stack.length>0};
}
function safeCode(error){return ['invalid_bundle','invalid_frame','incompatible_abi','resource_limit','build_required','admission_refused','invalid_handle','selection_not_available','installation_required','execution_error','internal_failure'].includes(error?.message)?error.message:'internal_failure';}
async function start(){
  const el=id=>document.getElementById(id); let bundle,display,index=0,step=-1,generation=0,timer,worker,handle=0,next=1,running=false,transitioning=false,watchdog,limit=0,modelLimit=0,suiteOffset=0,resultOffset=0,resultDisplay; const pending=new Map(),urls=[];
  function cancelNavigation(){clearInterval(timer);timer=undefined;generation++;}
  function declaredSteps(){const card=display.scenarios[index];const value=card?.declaration.kind==='object'?card.declaration.value.find(([key])=>key==='steps')?.[1]:undefined;return value?.kind==='list'?value.value:[];}
  function advance(){const steps=declaredSteps();if(step+1<steps.length)step++;else if(index+1<display.scenarios.length){index++;step=-1;}else return false;limit=0;return true;}
  function render(){
    const card=display.scenarios[index];const first=Math.floor(index/200)*200;
    el('scenario').replaceChildren();display.scenarios.slice(first,first+200).forEach((entry,i)=>{const option=document.createElement('option');option.value=String(first+i);option.textContent=entry.id;el('scenario').append(option);});el('scenario').value=String(index);
    el('cards-back').disabled=first===0;el('cards-next').disabled=first+200>=display.scenarios.length;
    el('position').textContent=card?`${index+1}/${display.scenarios.length} — ${card.id} — ${step<0?'Complete scenario declaration':`Declared step ${step+1}/${declaredSteps().length}`}`:'No declared scenarios';
    const rendered=card?renderValue(step<0?card.declaration:declaredSteps()[step],limit):{text:'No declared scenarios',omitted:false};
    el('declaration').textContent=rendered.text;el('more').disabled=!rendered.omitted;
    const model=renderValue(display.model,modelLimit);el('model').textContent=model.text;el('model-more').disabled=!model.omitted;
    const suite=renderValue(display.suite,suiteOffset);el('provenance').textContent=`Selected original SHA256: ${display.selected_digest}\nParents: ${display.parent_digests.join(', ')}\n`+suite.text;el('suite-more').disabled=!suite.omitted;
    if(resultDisplay){const result=renderValue(resultDisplay,resultOffset);el('results').textContent=result.text;el('results-more').disabled=!result.omitted;el('results-back').disabled=resultOffset===0;}
  }
  function clearResult(){resultDisplay=undefined;resultOffset=0;el('results').replaceChildren();el('results-more').disabled=true;el('results-back').disabled=true;el('downloads').replaceChildren();for(const url of urls)URL.revokeObjectURL(url);urls.length=0;}
  function setDisplay(value){validatePresentation(value);clearResult();display=value;index=0;step=-1;limit=0;modelLimit=0;suiteOffset=0;render();}
  function send(opcode,payload){const id=next++;return new Promise((resolve,reject)=>{pending.set(id,{resolve,reject});worker.postMessage({kind:'request',id,opcode,payload},[payload.buffer]);});}
  function discard(code){clearTimeout(watchdog);clearResult();worker?.terminate();worker=undefined;handle=0;running=false;for(const p of pending.values())p.reject(failure(code));pending.clear();el('runtime-state').textContent=code;el('run').disabled=true;el('select').disabled=true;el('restore').disabled=true;el('abort').disabled=true;}
  async function loadOriginal(){
    const w=new Writer();w.bytes(bundle.original);w.word(bundle.blobs.length);for(const b of bundle.blobs){w.string(b.path);w.bytes(b.bytes);}
    const loaded=await send(1,w.finish());handle=loaded.handle;setDisplay(loaded.display);el('runtime-state').textContent='Rust admission complete. Not executed.';el('run').textContent='Run all';el('run').disabled=false;el('select').disabled=bundle.manifest.execution.kind!=='coverage_input';el('restore').disabled=true;
  }
  async function transition(call){if(running||transitioning)throw failure('execution_error');transitioning=true;el('run').disabled=true;el('select').disabled=true;try{return await call();}catch(error){if(worker)discard(safeCode(error));throw error;}finally{transitioning=false;el('run').disabled=!handle;el('select').disabled=!handle||bundle.manifest.execution.kind!=='coverage_input';}}
  async function connect(){return transition(async()=>{
    cancelNavigation();discard('Connecting Rust runtime');worker=new Worker('worker.js',{type:'module'});el('abort').disabled=false;
    worker.onmessage=({data})=>{if(data.kind==='fatal'){discard(data.code);return;}const p=pending.get(data.id);if(!p)return;pending.delete(data.id);if(data.error){discard(data.error);p.reject(failure(data.error));}else p.resolve(data);};
    worker.onerror=event=>{event.preventDefault();discard('execution_error');};
    try{await loadOriginal();}finally{el('abort').disabled=true;}
  });}
  async function run(){
    if(!worker||!handle||running||transitioning)throw failure('installation_required');clearResult();running=true;el('run').disabled=true;el('select').disabled=true;el('abort').disabled=false;
    const runGeneration=generation;const w=new Writer();w.word(handle);w.raw(crypto.getRandomValues(new Uint8Array(16)));w.word(runGeneration);
    watchdog=setTimeout(()=>discard('aborted — cleanup unconfirmed'),300000);
    try{const result=await send(3,w.finish());clearTimeout(watchdog);
      if(generation!==runGeneration){el('runtime-state').textContent='Completed earlier navigation generation; result not attached to current selection.';return result;}
      const budget={nodes:0};validateValue(result.display,budget);resultDisplay=result.display;resultOffset=0;render();
      el('runtime-state').textContent=`Completed Rust run for ${result.digest}`;el('downloads').replaceChildren();for(const u of urls)URL.revokeObjectURL(u);urls.length=0;
      for(const [name,bytes] of [['report.json',result.report],['run.json',result.run]]){const url=URL.createObjectURL(new Blob([bytes],{type:'application/json'}));urls.push(url);const a=document.createElement('a');a.href=url;a.download=name;a.textContent=`Download ${name} `;el('downloads').append(a);}return result;
    }catch(error){if(worker)discard(safeCode(error));throw error;}finally{clearTimeout(watchdog);running=false;el('abort').disabled=true;el('run').disabled=!handle;el('select').disabled=!handle||bundle.manifest.execution.kind!=='coverage_input';}
  }
  function action(button,call){el(button).onclick=()=>Promise.resolve().then(call).catch(error=>{if(error?.message==='aborted — cleanup unconfirmed')return;el('error').textContent=safeCode(error);});}
  try{
    bundle=await acquire();setDisplay(bundle.display);document.body.dataset.ready='true';el('state').textContent='Declarations admitted at emission. Not executed.';
    action('back',()=>{cancelNavigation();if(step>=0)step--;else if(index>0){index--;step=declaredSteps().length-1;}limit=0;render();});action('step',()=>{cancelNavigation();advance();render();});
    action('reset',()=>{cancelNavigation();index=0;step=-1;limit=0;render();});action('play',()=>{cancelNavigation();timer=setInterval(()=>{if(!advance())clearInterval(timer);render();},750);});
    el('scenario').onchange=()=>{cancelNavigation();index=Number(el('scenario').value);step=-1;limit=0;render();};
    action('more',()=>{limit+=DISPLAY_PAGE;render();});action('declaration-back',()=>{limit=Math.max(0,limit-DISPLAY_PAGE);render();});
    action('model-more',()=>{modelLimit+=DISPLAY_PAGE;render();});action('model-back',()=>{modelLimit=Math.max(0,modelLimit-DISPLAY_PAGE);render();});
    action('suite-more',()=>{suiteOffset+=DISPLAY_PAGE;render();});action('suite-back',()=>{suiteOffset=Math.max(0,suiteOffset-DISPLAY_PAGE);render();});
    action('results-more',()=>{resultOffset+=DISPLAY_PAGE;render();});action('results-back',()=>{resultOffset=Math.max(0,resultOffset-DISPLAY_PAGE);render();});
    action('cards-back',()=>{cancelNavigation();index=Math.max(0,Math.floor(index/200)*200-200);step=-1;limit=0;render();});action('cards-next',()=>{cancelNavigation();index=Math.min(display.scenarios.length-1,Math.floor(index/200)*200+200);step=-1;limit=0;render();});action('connect',connect);action('run',run);
    action('select',()=>transition(async()=>{cancelNavigation();const w=new Writer();w.word(handle);w.word(1);w.string(display.scenarios[index].id);const selected=await send(2,w.finish());handle=selected.handle;setDisplay(selected.display);el('run').textContent='Run selected coverage';el('restore').disabled=false;el('runtime-state').textContent='Coverage selection admitted. Not executed.';}));
    action('restore',()=>transition(async()=>{if(!worker)throw failure('execution_error');cancelNavigation();await loadOriginal();}));
    action('abort',()=>discard('aborted — cleanup unconfirmed'));
    window.essBrowser=Object.freeze({connect,run}); // Actual product controls, also driven by BiDi.
  }catch(error){document.body.dataset.error='true';el('state').textContent='Presentation refused';el('error').textContent=safeCode(error);el('connect').disabled=true;}
}
if(typeof document!=='undefined')start();
