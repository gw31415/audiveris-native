// SPDX-License-Identifier: AGPL-3.0-or-later
import java.lang.reflect.*;
import java.nio.file.*;
import java.util.*;
import java.awt.Rectangle;
public class SlurLifecycleObserver extends NativeSlurCandidateMathProbe {
 public static List<String> transformedMethods;
 static volatile boolean recording=false;
 static Object sheet;
 static final List<Object> events=new ArrayList<>(), errors=new ArrayList<>(), held=new ArrayList<>();
 static final IdentityHashMap<Object,String> identities=new IdentityHashMap<>();
 static final IdentityHashMap<Object,Object> sourceOf=new IdentityHashMap<>(),targetOf=new IdentityHashMap<>();
 static synchronized String label(Object x){if(x==null)return null;return identities.computeIfAbsent(x,k->"object-"+(identities.size()+1)+":"+x.getClass().getSimpleName());}
 static Object id(Object x)throws Exception{return x==null?null:call(x,"getId");}
 static Object raw(Object x,String name)throws Exception{return x==null?null:field(x,name);}
 static boolean contains(Object xs,Object x){if(xs instanceof Collection<?> c)for(Object item:c)if(item==x)return true;return false;}
 static Object bounds(Object x)throws Exception{Rectangle r=(Rectangle)raw(x,"bounds");return r==null?null:List.of(r.x,r.y,r.width,r.height);}
 static Object scalar(Object x){return x==null?null:x.toString();}
 static Object chord(Object x)throws Exception {if(x==null)return null;Object measure=field(x,"measure"),voice=field(x,"voice"),slot=field(x,"slot");return map("id",id(x),"object",label(x),"bounds_raw",bounds(x),"manual",call(x,"isManual"),"part_raw",label(field(x,"part")),"measure",label(measure),"measure_part",label(measure==null?null:call(measure,"getPart")),"stack",label(measure==null?null:call(measure,"getStack")),"expected_duration",measure==null?null:scalar(call(call(measure,"getStack"),"getExpectedDuration")),"voice",label(voice),"slot",label(slot),"time_offset",scalar(field(x,"timeOffset")));}
 static Object head(Object x)throws Exception {if(x==null)return null;Object staff=field(x,"staff"),part=staff==null?null:call(staff,"getPart"),chord=call(x,"getChord");return map("id",id(x),"object",label(x),"bounds_raw",bounds(x),"manual",call(x,"isManual"),"pitch",field(x,"pitch"),"staff",label(staff),"staff_id",id(staff),"part",label(part),"part_id",id(part),"chord",chord(chord));}
 static Object relation(Object edge,Object source,Object target)throws Exception {return map("object",label(edge),"kind",edge.getClass().getSimpleName(),"source",label(source),"target",head(target),"side",scalar(call(edge,"getSide")),"manual",call(edge,"isManual"),"grade",call(edge,"getGrade"),"source_ratio",call(edge,"getSourceRatio"),"target_ratio",call(edge,"getTargetRatio"),"euclidean",call(edge,"getEuclidean"));}
 static Object state(Object slur)throws Exception {
  Object sig=field(slur,"sig"),part=field(slur,"part");List<Object> edges=new ArrayList<>(),memberships=new ArrayList<>();
  if(sig!=null)for(Object e:(Set<?>)call(sig,"edgeSet"))if(e.getClass().getSimpleName().equals("SlurHeadRelation")&&call(sig,"getEdgeSource",e)==slur)edges.add(relation(e,slur,call(sig,"getEdgeTarget",e)));
  for(Object sys:(List<?>)call(sheet,"getSystems"))for(Object p:(List<?>)call(sys,"getParts")){Object xs=field(p,"slurs");if(contains(xs,slur))memberships.add(map("system",id(sys),"part",id(p),"part_object",label(p),"collection_object",label(xs),"identity_occurrences",((Collection<?>)xs).stream().filter(x->x==slur).count()));}
  return map("object",label(slur),"id",id(slur),"sig",label(sig),"system",sig==null?null:id(call(sig,"getSystem")),"in_actual_SIG",sig!=null&&contains(call(sig,"vertexSet"),slur),"manual",call(slur,"isManual"),"tie",call(slur,"isTie"),"abnormal",call(slur,"isAbnormal"),"removed",call(slur,"isRemoved"),"part_raw",label(part),"part_id_raw",id(part),"glyph",label(call(slur,"getGlyph")),"glyph_id",id(call(slur,"getGlyph")),"curve",bezier(call(slur,"getCurve")),"live_head_relations",edges,"actual_part_memberships",memberships);
 }
 static List<Object> slurs()throws Exception {List<Object> out=new ArrayList<>();for(Object sys:(List<?>)call(sheet,"getSystems"))for(Object v:(Set<?>)call(call(sys,"getSig"),"vertexSet"))if(v.getClass().getSimpleName().equals("SlurInter"))out.add(v);return out;}
 static List<Object> allStates()throws Exception{List<Object> out=new ArrayList<>();for(Object s:slurs())out.add(state(s));return out;}
 static List<Object> trace(){List<Object> out=new ArrayList<>();for(StackTraceElement x:Thread.currentThread().getStackTrace())if(x.getClassName().startsWith("org.audiveris.omr."))out.add(map("class",x.getClassName(),"method",x.getMethodName(),"line",x.getLineNumber()));return out;}
 public static synchronized void capture(String hook,Object receiver,Object arg){
  if(!recording)return;
  if(hook.startsWith("sig/inter/AbstractInter.setPart")&&!receiver.getClass().getSimpleName().equals("SlurInter"))return;
  try {
   Map<String,Object> row=map("order",events.size()+1,"hook",hook,"receiver",label(receiver),"argument",label(arg),"call_stack",trace());
   if(hook.contains("dispatchToParts")){row.put("all_slurs",allStates());}
   else if(hook.contains("doLink")){Object s=field(arg,"slur");if(!contains(held,s))held.add(s);List<Object> links=new ArrayList<>();for(var e:((Map<?,?>)field(arg,"links")).entrySet()){Object link=e.getValue();if(link!=null){Object rel=field(link,"relation"),h=field(link,"partner");sourceOf.put(rel,s);targetOf.put(rel,h);links.add(map("map_side",scalar(e.getKey()),"link_object",label(link),"relation",relation(rel,s,h)));}}row.put("pending_links",links);row.put("slur",state(s));}
   else if(hook.startsWith("sig/relation/SlurHeadRelation")){Object s=call(arg,"getEdgeSource"),h=call(arg,"getEdgeTarget");row.put("relation",relation(receiver,s,h));row.put("event_edge_is_receiver",call(arg,"getEdge")==receiver);row.put("same_pending_relation",sourceOf.get(receiver)==s&&targetOf.get(receiver)==h);row.put("slur",state(s));}
   else if(hook.startsWith("sheet/Part")){row.put("part_id",id(receiver));row.put("part_collection",label(field(receiver,"slurs")));row.put("part_contains_same_slur",contains(field(receiver,"slurs"),arg));row.put("slur",state(arg));}
   else {row.put("slur",state(receiver));}
   events.add(row);
  }catch(Throwable t){errors.add(map("hook",hook,"error",exception(t)));}
 }
 static Object ownerInventory()throws Exception{List<Object> out=new ArrayList<>();for(Object sys:(List<?>)call(sheet,"getSystems"))for(Object x:(Set<?>)call(call(sys,"getSig"),"vertexSet"))if(x.getClass().getSimpleName().equals("HeadChordInter"))out.add(chord(x));return out;}
 public static void main(String[] args)throws Exception {
  Object rootLogger=Class.forName("org.slf4j.LoggerFactory").getMethod("getLogger",String.class).invoke(null,"ROOT");call(rootLogger,"setLevel",Class.forName("ch.qos.logback.classic.Level").getField("INFO").get(null));
  transformedMethods=Files.readAllLines(Path.of(args[2]));Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));Object stub=((List<?>)call(book,"getStubs")).get(0);sheet=call(stub,"getSheet");
  if(!call(stub,"getLatestStep").toString().equals("CHORDS")||!slurs().isEmpty())throw new IllegalStateException("requires unchanged pre-CURVES CHORDS Book with no Slurs");
  Object beforeOwners=ownerInventory(),before=allStates();List<Object> preFill=new ArrayList<>();for(Object sys:(List<?>)call(sheet,"getSystems"))for(Object part:(List<?>)call(sys,"getParts"))for(Object measure:(List<?>)call(part,"getMeasures"))preFill.add(map("system",id(sys),"part",id(part),"measure",label(measure),"clefs_count",((Collection<?>)call(measure,"getClefs")).size(),"keys_count",((Collection<?>)field(measure,"keys")).size()));recording=true;
  Object step=c("sheet.curve.CurvesStep").getConstructor().newInstance();call(step,"doit",sheet);
  recording=false;Object after=allStates(),afterOwners=ownerInventory();
  Files.writeString(Path.of(args[1]),json(map("scope","Fresh original CURVES automatic construction from an unchanged persisted original CHORDS checkpoint; upstream owners are unmarshalled, not same allocations as prior image run. CURVES is invoked once in memory via untouched original CurvesStep.doit. No upstream image recognition rerun, replay of existing Slurs, native input, Book storage or original file mutation.","instrumentation","An offline observation-only class overlay injects observational entry/normal-return calls into six named methods; original operation bodies/control flow/parameters are retained. It does not substitute any Slur/Head/Chord/Part/collection/relation owner or listener. Observations use raw part/clock fields and actual identity membership; getPart is never called on Slur. Observer errors are recorded and do not change original returns.","latest_step_before_and_after",call(stub,"getLatestStep"),"transformed_methods",transformedMethods,"after_reload_measure_memberships",preFill,"before_slurs",before,"before_chord_owners",beforeOwners,"events",events,"after_slurs",after,"after_chord_owners",afterOwners,"observer_errors",errors))+"\n");
  if(!errors.isEmpty())throw new IllegalStateException("observer errors: "+errors);
 }
}
