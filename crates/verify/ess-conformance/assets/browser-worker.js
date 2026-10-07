// Synchronous Rust execution is isolated from declaration navigation; no target transport lives here.
import {MAX_FRAME,Reader,fetchBytes,request,failure,text,parseControl} from './player.js';
let instance;
async function runtime(){if(instance)return instance;const bytes=await fetchBytes('runner.wasm',MAX_FRAME);const loaded=await WebAssembly.instantiate(bytes,{});instance=loaded.instance.exports;if(instance.ess_browser_abi_version()!==0x00010000)throw failure('incompatible_abi');return instance;}
function response(bytes,id){
  const r=new Reader(bytes);if(text(r.raw(4))!=='ESBW'||r.word()!==1||r.word()!==0)throw failure('incompatible_abi');const tag=r.word();if(r.word()!==id)throw failure('invalid_frame');const status=r.word(),length=r.word();if(length!==bytes.length-28)throw failure('invalid_frame');
  if(tag===5){const code=r.string(),location=r.string();r.end();const codes=['invalid_frame','incompatible_abi','invalid_bundle','admission_refused','invalid_handle','selection_not_available','installation_required','resource_limit','execution_error','internal_failure'];if(status<1||status>codes.length||codes[status-1]!==code||location!=='$browser')throw failure('invalid_frame');return {id,error:code};}if(status!==0)throw failure('invalid_frame');
  if(tag===1||tag===2){const handle=r.word(),digest=r.string(),display=parseControl(r.blob());r.end();if(handle===0)throw failure('invalid_frame');return {id,handle,digest,display};}
  if(tag===3){const digest=r.string(),nonce=r.raw(16),generation=r.word(),report=r.blob(),run=r.blob(),display=parseControl(r.blob());r.end();return {id,digest,nonce,generation,report,run,display};}
  if(tag===4){r.end();return {id};}throw failure('invalid_frame');
}
let queue=Promise.resolve();
self.onmessage=({data})=>{queue=queue.then(async()=>{
  try{
    if(data.kind!=='request'||!(data.payload instanceof Uint8Array))throw failure('invalid_frame');
    const host=await runtime(),frame=request(data.opcode,data.id,data.payload),address=host.ess_browser_reserve(frame.length);
    if(!address)throw failure('resource_limit');if(address+frame.length>host.memory.buffer.byteLength)throw failure('invalid_frame');
    new Uint8Array(host.memory.buffer,address,frame.length).set(frame);const pointer=host.ess_browser_dispatch(frame.length),length=host.ess_browser_response_len();
    if(!pointer||length>MAX_FRAME||pointer+length>host.memory.buffer.byteLength)throw failure('invalid_frame');
    const result=response(new Uint8Array(host.memory.buffer,pointer,length).slice(),data.id);self.postMessage(result);
  }catch(error){const code=['build_required','invalid_frame','incompatible_abi','resource_limit'].includes(error?.message)?error.message:'execution_error';self.postMessage({kind:'fatal',code});}
});};
