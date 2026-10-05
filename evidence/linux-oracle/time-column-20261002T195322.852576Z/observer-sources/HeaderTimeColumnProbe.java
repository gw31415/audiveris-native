// SPDX-License-Identifier: AGPL-3.0-or-later
// Untouched original HeaderTimeColumn.retrieveTime; Map/List observation only.
import org.audiveris.omr.sheet.Book;import java.util.*;import java.nio.file.*;import java.lang.reflect.*;
public class HeaderTimeColumnProbe extends KeyBuilderProcessProbe {
 final Object book;Object system,column;Map<Object,Object> builders;boolean recording;final List<String> events=new ArrayList<>();
 public HeaderTimeColumnProbe(Book b,SortedSet<Integer> ids){super(b,ids);book=b;}
 static String timeInter(Object value)throws Exception{
  if(value==null)return "null";String base=inter(value);String extra=",\"frozen\":"+call(value,"isFrozen");String kind=value.getClass().getSimpleName();
  if(kind.equals("TimeNumberInter"))extra+=",\"value\":"+call(value,"getValue");
  if(Class.forName("org.audiveris.omr.sig.inter.AbstractTimeInter").isInstance(value))extra+=",\"value\":"+q(call(value,"getValue"))+",\"numerator\":"+call(value,"getNumerator")+",\"denominator\":"+call(value,"getDenominator");
  if(kind.equals("TimePairInter")){List<String> members=new ArrayList<>();for(Object x:(Collection<?>)call(value,"getMembers"))members.add(timeInter(x));extra+=",\"members\":["+String.join(",",members)+"]";}
  return base.substring(0,base.length()-1)+extra+"}";
 }
 static String numbers(Object params)throws Exception{List<String> vals=new ArrayList<>();for(Field f:params.getClass().getDeclaredFields()){f.setAccessible(true);if(!Modifier.isStatic(f.getModifiers())&&f.get(params)instanceof Number)vals.add(q(f.getName())+":"+f.get(params));}return "{"+String.join(",",vals)+"}";}
 static String inters(Collection<?> list)throws Exception{List<String> vals=new ArrayList<>();for(Object x:list)vals.add(timeInter(x));return "["+String.join(",",vals)+"]";}
 String builder(Object b)throws Exception{
  Object projection=get(b,"projection");int min=(Integer)call(projection,"getXMin"),max=(Integer)call(projection,"getXMax");List<Integer> values=new ArrayList<>();for(int x=min;x<=max;x++)values.add((Integer)call(projection,"getValue",x));
  List<String> adapters=new ArrayList<>();for(var e:((Map<?,?>)get(b,"adapters")).entrySet()){
   Object a=e.getValue();Object attachment=((Map<?,?>)call(get(b,"staff"),"getAttachments")).get(e.getKey().toString().equals("WHOLE")?"TF":e.getKey().toString().equals("NUM")?"TN":"TD");List<?> parts=(List<?>)call(a,"getParts");List<String> glyphs=new ArrayList<>();for(Object g:parts)glyphs.add(canonicalGlyph(g));
   List<String> best=new ArrayList<>();for(var x:((Map<?,?>)get(a,"bestMap")).entrySet())best.add("{\"shape\":"+q(x.getKey())+",\"inter\":"+timeInter(x.getValue())+"}");
   Object graph=get(a,"graph");List<String> links=new ArrayList<>();for(Object link:(Collection<?>)call(graph,"edgeSet"))links.add("["+parts.indexOf(call(graph,"getEdgeSource",link))+","+parts.indexOf(call(graph,"getEdgeTarget",link))+"]");
   adapters.add("{\"kind\":"+q(e.getKey())+",\"search_rect\":"+rect(attachment)+",\"trials\":"+get(a,"trials")+",\"parts\":["+String.join(",",glyphs)+"],\"links\":["+String.join(",",links)+"],\"best_per_shape\":["+String.join(",",best)+"]}");
  }
  List<String> spaces=new ArrayList<>();for(Object x:(List<?>)call(b,"getSpaces"))spaces.add("["+get(x,"start")+","+get(x,"stop")+"]");
  return "{\"range\":"+range(get(b,"range"))+",\"roi\":"+rect(get(b,"roi"))+",\"params\":"+numbers(get(b,"params"))+",\"projection\":{\"x_min\":"+min+",\"x_max\":"+max+",\"values\":"+values+"},\"spaces\":["+String.join(",",spaces)+"],\"adapters\":["+String.join(",",adapters)+"],\"wholes\":"+inters((Collection<?>)get(b,"wholes"))+",\"nums\":"+inters((Collection<?>)get(b,"nums"))+",\"dens\":"+inters((Collection<?>)get(b,"dens"))+",\"selected_time\":"+timeInter(get(b,"timeInter"))+"}";
 }
 String state()throws Exception{
  List<String> staffs=new ArrayList<>(),vertices=new ArrayList<>(),relations=new ArrayList<>();Object sig=call(system,"getSig");
  for(Object st:(List<?>)call(system,"getStaves")){
   Object h=call(st,"getHeader"),b=builders.get(st);staffs.add("{\"staff_id\":"+call(st,"getId")+",\"header_start\":"+call(st,"getHeaderStart")+",\"header_stop\":"+call(st,"getHeaderStop")+",\"clef_stop\":"+call(st,"getClefStop")+",\"key_stop\":"+call(st,"getKeyStop")+",\"time_stop\":"+call(st,"getTimeStop")+",\"header_time\":"+timeInter(get(h,"time"))+",\"time_range\":"+(get(h,"timeRange")==null?"null":range(get(h,"timeRange")))+",\"builder\":"+(b==null?"null":builder(b))+"}");
  }
  for(Object v:(Collection<?>)call(sig,"vertexSet"))vertices.add(timeInter(v));
  for(Object e:(Collection<?>)call(sig,"edgeSet")){String cause=e.getClass().getSimpleName().equals("Exclusion")?",\"cause\":"+q(get(e,"cause")):"";relations.add("{\"kind\":"+q(e.getClass().getSimpleName())+",\"source\":"+call(call(sig,"getEdgeSource",e),"getId")+",\"target\":"+call(call(sig,"getEdgeTarget",e),"getId")+cause+"}");}
  return "{\"time_value\":"+q(get(column,"timeValue"))+",\"staffs\":["+String.join(",",staffs)+"],\"sig_inters\":["+String.join(",",vertices)+"],\"relations\":["+String.join(",",relations)+"]}";
 }
 void observe(String op,Object staff){if(recording)return;recording=true;try{List<String> stack=new ArrayList<>();for(var e:Thread.currentThread().getStackTrace())if(e.getClassName().startsWith("org.audiveris.omr.sheet.time."))stack.add(q(e.getClassName()+"."+e.getMethodName()+":"+e.getLineNumber()));events.add("{\"index\":"+events.size()+",\"operation\":"+q(op)+",\"staff_id\":"+(staff==null?"null":call(staff,"getId"))+",\"original_call_stack\":["+String.join(",",stack)+"],\"state\":"+state()+"}");}catch(Exception e){throw new RuntimeException(e);}finally{recording=false;}}
 class ObservedList extends AbstractList<Object>{final List<Object> original;final Object staff;boolean filtered;ObservedList(List<Object> o,Object st){original=o;staff=st;}public Object get(int i){return original.get(i);}public int size(){return original.size();}public boolean add(Object v){return original.add(v);}public Object remove(int i){return original.remove(i);}public Object set(int i,Object v){return original.set(i,v);}public Iterator<Object> iterator(){if(!recording&&!filtered)for(var e:Thread.currentThread().getStackTrace())if(e.getMethodName().equals("filterCandidates")){filtered=true;observe("before_filterCandidates",staff);break;}return original.iterator();}}
 class ObservedAdapters extends AbstractMap<Object,Object>{final Map<Object,Object> original;final Object staff;ObservedAdapters(Map<Object,Object> o,Object st){original=o;staff=st;}public Set<Entry<Object,Object>> entrySet(){return original.entrySet();}public Object get(Object k){observe("before_adapters_get_"+k,staff);return original.get(k);}public Object put(Object k,Object v){observe("before_adapter_put_"+k,staff);return original.put(k,v);}}
 class ObservedBuilders extends AbstractMap<Object,Object>{int pass;public Set<Entry<Object,Object>> entrySet(){return builders.entrySet();}public Object get(Object k){observe("before_builder_get",k);return builders.get(k);}
  @SuppressWarnings("unchecked") public Object put(Object k,Object v){try{HeaderTimeColumnProbe.put(v,"adapters",new ObservedAdapters((Map<Object,Object>)HeaderTimeColumnProbe.get(v,"adapters"),k));HeaderTimeColumnProbe.put(v,"nums",new ObservedList((List<Object>)HeaderTimeColumnProbe.get(v,"nums"),k));}catch(Exception e){throw new RuntimeException(e);}return builders.put(k,v);}
  public Collection<Object> values(){int p=++pass;observe("before_builders_values_"+p,null);return new AbstractCollection<>(){public int size(){return builders.size();}public Iterator<Object> iterator(){Iterator<Entry<Object,Object>> it=builders.entrySet().iterator();return new Iterator<>(){Object staff;boolean pending;public boolean hasNext(){if(pending){observe("after_builder_iteration_"+p,staff);pending=false;}return it.hasNext();}public Object next(){var e=it.next();staff=e.getKey();pending=true;return e.getValue();}};}};}
 }
 static String bestSignature(Object adapter)throws Exception {List<String> out=new ArrayList<>();for(var e:((Map<?,?>)get(adapter,"bestMap")).entrySet()){Object v=e.getValue();out.add(q(e.getKey())+":"+call(v,"getGrade")+":"+canonicalGlyph(call(v,"getGlyph")));}return String.join(";",out);}
 String replayCandidates(List<Map<Object,Object>> contexts)throws Exception {
  List<String> records=new ArrayList<>();Class<?> adapterType=Class.forName("org.audiveris.omr.glyph.GlyphCluster$GlyphAdapter"),builderType=Class.forName("org.audiveris.omr.sheet.time.HeaderTimeBuilder"),kindType=Class.forName("org.audiveris.omr.sheet.time.TimeBuilder$TimeKind");
  Object classifier=Class.forName("org.audiveris.omr.classifier.ShapeClassifier").getMethod("getInstance").invoke(null);double minGrade=(Double)Class.forName("org.audiveris.omr.glyph.Grades").getField("timeMinGrade").get(null);double ratio=(Double)Class.forName("org.audiveris.omr.glyph.Grades").getField("intrinsicRatio").get(null);
  for(Map<Object,Object> context:contexts)for(var entry:context.entrySet()){
   Object staff=entry.getKey(),b=entry.getValue();for(var aentry:((Map<?,?>)get(b,"adapters")).entrySet()){
    Object original=aentry.getValue(),kind=aentry.getKey();List<?> parts=(List<?>)call(original,"getParts");boolean whole=kind.toString().equals("WHOLE");Class<?> cls=Class.forName("org.audiveris.omr.sheet.time.HeaderTimeBuilder$"+(whole?"WholeAdapter":"HalfAdapter"));Constructor<?> ctor=whole?cls.getDeclaredConstructor(builderType,List.class):cls.getDeclaredConstructor(builderType,kindType,List.class);ctor.setAccessible(true);Object adapter=whole?ctor.newInstance(b,parts):ctor.newInstance(b,kind,parts);List<String> trials=new ArrayList<>();
    Object proxy=Proxy.newProxyInstance(adapterType.getClassLoader(),new Class<?>[]{adapterType},(pr,method,args)->{
     Object result=method.invoke(adapter,args);if(method.getName().equals("evaluateGlyph")){
      Object glyph=args[0];List<Integer> membership=new ArrayList<>();for(Object part:(Set<?>)args[1])membership.add(parts.indexOf(part));Collections.sort(membership);List<String> evals=new ArrayList<>();Object evalArray=classifier.getClass().getMethod("evaluate",Class.forName("org.audiveris.omr.glyph.Glyph"),int.class,int.class,double.class,EnumSet.class).invoke(classifier,glyph,call(staff,"getSpecificInterline"),get(get(b,"params"),"maxEvalRank"),minGrade/ratio,null);for(int i=0;i<Array.getLength(evalArray);i++)evals.add(eval(Array.get(evalArray,i)));java.awt.Point centroid=(java.awt.Point)call(glyph,"getCentroid");
      trials.add("{\"glyph\":"+canonicalGlyph(glyph)+",\"part_indices\":"+membership+",\"centroid\":["+centroid.x+","+centroid.y+"],\"centroid_pitch\":"+call(staff,"pitchPositionOf",centroid)+",\"evaluations\":["+String.join(",",evals)+"]}");
     }return result;
    });
    Object cluster=Class.forName("org.audiveris.omr.glyph.GlyphCluster").getConstructor(adapterType,Class.forName("org.audiveris.omr.glyph.GlyphGroup")).newInstance(proxy,null);call(cluster,"decompose");boolean equal=bestSignature(adapter).equals(bestSignature(original))&&get(adapter,"trials").equals(get(original,"trials"));if(!equal)throw new IllegalStateException("Original adapter replay differs from completed pipeline");
    records.add("{\"system_id\":"+call(call(staff,"getSystem"),"getId")+",\"staff_id\":"+call(staff,"getId")+",\"kind\":"+q(kind)+",\"original_best_map_and_trial_count_equal\":true,\"minimum_raw_grade\":"+(minGrade/ratio)+",\"trials\":["+String.join(",",trials)+"]}");
   }
  }return "["+String.join(",",records)+"]";
 }
 @SuppressWarnings("unchecked") public void process(){try{
  Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");call(Class.forName("org.audiveris.omr.sheet.grid.GridStep").getConstructor().newInstance(),"doit",sheet);List<String> out=new ArrayList<>();List<Map<Object,Object>> contexts=new ArrayList<>();
  for(Object sys:(List<?>)call(sheet,"getSystems")){
   system=sys;events.clear();Object hb=Class.forName("org.audiveris.omr.sheet.header.HeaderBuilder").getConstructor(Class.forName("org.audiveris.omr.sheet.SystemInfo")).newInstance(system);call(hb,"computeHeaderStarts");int clef=(Integer)call(get(hb,"clefColumn"),"retrieveClefs");call(hb,"setSystemClefStop",clef);int key=(Integer)call(get(hb,"keyColumn"),"retrieveKeys",get(hb,"maxHeaderWidth"));call(hb,"setSystemKeyStop",key);
   column=get(hb,"timeColumn");builders=(Map<Object,Object>)get(column,"builders");if(!Boolean.getBoolean("native.probe.plainTimeColumn"))put(column,"builders",new ObservedBuilders());int offset=(Integer)call(column,"retrieveTime");String after=state();contexts.add(builders);call(hb,"setSystemTimeStop",offset);
   out.add("{\"system_id\":"+call(system,"getId")+",\"clef_offset\":"+clef+",\"key_offset\":"+key+",\"returned_time_offset\":"+offset+",\"after_retrieve_time\":"+after+",\"after_system_time_stop\":"+state()+",\"chronology\":["+String.join(",",events)+"]}");
  }
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"stage\":\"HeaderTimeColumn_retrieveTime_and_system_time_stop_before_final_clef_selection_bar_purge_header_freeze\",\"passive_collection_hooks\":"+!Boolean.getBoolean("native.probe.plainTimeColumn")+",\"systems\":["+String.join(",",out)+"],\"candidate_replay_after_all_systems\":"+replayCandidates(contexts)+"}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
