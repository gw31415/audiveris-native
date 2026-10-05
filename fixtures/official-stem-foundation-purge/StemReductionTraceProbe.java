// SPDX-License-Identifier: AGPL-3.0-or-later
// Original-only reduction trace, preserving Java check order; no native inputs.
import org.audiveris.omr.sheet.Book;
import java.util.*;
import java.nio.file.*;
import java.lang.reflect.*;
import java.util.function.Predicate;
public class StemReductionTraceProbe extends StemImageScheduleProbe {
 private final Object book;
 public StemReductionTraceProbe(Book b, SortedSet<Integer> ids){super(b,ids);book=b;}
 static String detailed(Object sys)throws Exception{
  Object sig=call(sys,"getSig");List<String> nodes=new ArrayList<>(), exclusions=new ArrayList<>();
  for(Object v:(Collection<?>)call(sig,"vertexSet"))if(v.getClass().getSimpleName().equals("HeadInter")||v.getClass().getSimpleName().equals("StemInter"))
   nodes.add("{\"kind\":\""+v.getClass().getSimpleName()+"\",\"bounds\":"+rect(call(v,"getBounds"))+",\"grade\":"+call(v,"getGrade")+",\"contextual_grade\":"+call(v,"getContextualGrade")+"}");
  for(Object e:(Collection<?>)call(sig,"edgeSet"))if(e.getClass().getSimpleName().equals("Exclusion")){
   Object a=call(sig,"getEdgeSource",e),b=call(sig,"getEdgeTarget",e);
   exclusions.add("{\"source_kind\":\""+a.getClass().getSimpleName()+"\",\"target_kind\":\""+b.getClass().getSimpleName()+"\",\"source_bounds\":"+rect(call(a,"getBounds"))+",\"target_bounds\":"+rect(call(b,"getBounds"))+"}");
  }
  return "{\"state\":"+state(sys)+",\"nodes\":["+String.join(",",nodes)+"],\"exclusions\":["+String.join(",",exclusions)+"]}";
 }
 static void record(List<String> out,Object sys,String stage,Object changes)throws Exception{
  out.add("{\"stage\":\""+stage+"\",\"changes\":"+(changes instanceof Collection<?> ? ((Collection<?>)changes).size():changes)+",\"detail\":"+detailed(sys)+"}");
 }
 @SuppressWarnings("unchecked") public void process(){try{
  Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");List<String> systems=new ArrayList<>();
  for(Object sys:(List<?>)call(sheet,"getSystems")){
   Object retr=inspect(sys,sheet);call(retr,"linkStems");call(retr,"finalizeStems");Object sig=call(sys,"getSig");
   Object reducer=Class.forName("org.audiveris.omr.sig.SigReducer").getConstructor(Class.forName("org.audiveris.omr.sheet.SystemInfo"),boolean.class).newInstance(sys,true);
   Constructor<?> ctor=Class.forName("org.audiveris.omr.sig.SigReducer$AdapterForFoundations").getDeclaredConstructors()[0];ctor.setAccessible(true);Object adapter=ctor.newInstance(reducer);
   List<String> stages=new ArrayList<>();record(stages,sys,"finalized",0);
   List<?> headers=(List<?>)call(reducer,"getHeadersInters");Field field=reducer.getClass().getDeclaredField("overlapPredicate");field.setAccessible(true);Predicate<Object> predicate=(Predicate<Object>)field.get(null);
   List<Object> filtered=new ArrayList<>();for(Object v:(Collection<?>)call(sig,"vertexSet"))if(predicate.test(v)&&!headers.contains(v)&&!(Boolean)call(v,"isRemoved")&&!(Boolean)call(v,"isImplicit"))filtered.add(v);
   filtered.sort(Comparator.comparingInt(v->{try{return ((java.awt.Rectangle)call(v,"getBounds")).x;}catch(Exception e){throw new RuntimeException(e);}}));
   call(reducer,"detectOverlaps",filtered);record(stages,sys,"overlaps",0);call(adapter,"checkFrozens");call(sig,"contextualize");call(adapter,"prolog");record(stages,sys,"prolog",0);
   int epoch=0;boolean again;
   do{
    if(++epoch>30)throw new IllegalStateException("trace epoch limit");
    Object deleted=call(reducer,"contextualizeAndPurge");int del=((Collection<?>)deleted).size();record(stages,sys,epoch+".purge",deleted);call(adapter,"checkSlurs");
    int iteration=0,mods;
    do{
     if(++iteration>30)throw new IllegalStateException("trace consistency limit");mods=0;
     for(String name:new String[]{"checkStemEndingHeads","checkHeads","checkHooksHaveStem","checkBeamsHaveBothStems","checkLedgers","checkStems"}){
      Object change=call(reducer,name);mods+=(Integer)change;record(stages,sys,epoch+"."+iteration+"."+name,change);
      record(stages,sys,epoch+"."+iteration+".purgeAfter."+name,call(reducer,"contextualizeAndPurge"));
     }
    }while(mods>0);
    Object reduced=call(sig,"reduceExclusions");record(stages,sys,epoch+".reduceExclusions",reduced);
    int late=0;while((Integer)call(adapter,"checkLateConsistencies")>0)if(++late>30)throw new IllegalStateException("trace late limit");record(stages,sys,epoch+".late",late);
    again=del>0||!((Collection<?>)reduced).isEmpty();
   }while(again);
   systems.add("{\"system\":"+call(sys,"getId")+",\"stages\":["+String.join(",",stages)+"]}");
  }
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"native_outputs_used\":false,\"scope\":\"original_XML_loaded_reduction_order_trace\",\"systems\":["+String.join(",",systems)+"]}");
 }catch(Exception e){throw new RuntimeException(e);}}
}
