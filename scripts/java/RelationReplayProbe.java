// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;import java.util.*;import java.awt.Rectangle;import java.lang.reflect.*;import com.fasterxml.jackson.databind.ObjectMapper;
public class RelationReplayProbe extends TimedDotSupportProbe {
 public static void main(String[] args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);List<Object> cases=new ArrayList<>();
  for(boolean[] flags:List.of(new boolean[]{false,true},new boolean[]{true,true},new boolean[]{false,false})) {
   boolean oldManual=flags[0],newManual=flags[1];
   Object[] a=load(args[0]);Object rhythm=make("sheet.rhythm.RhythmsStep");call(make("sheet.rhythm.PageRhythm",call(a[2],"getPage")),"reprocessStack",a[4]);Object d=((List<?>)a[6]).get(0),old=null;
   for(Object e:(Collection<?>)call(a[3],"edgesOf",d))if(e.getClass().getSimpleName().equals("AugmentationRelation")){old=e;break;}
   Object h0=call(a[3],"getEdgeTarget",old),c0=call(h0,"getChord");List<Object> candidates=new ArrayList<>();for(Object ch:(Collection<?>)call(a[5],"getHeadChords"))if(ch!=c0&&call(ch,"getDurationSansDotOrTuplet").equals(call(c0,"getDurationSansDotOrTuplet")))candidates.add(ch);
   candidates.sort(Comparator.comparingInt(ch->{try{return ((Rectangle)call(ch,"getBounds")).x;}catch(Exception e){throw new RuntimeException(e);}}));Object c1=candidates.get(0),h1=((List<?>)call(c1,"getMembers")).get(0);
   if(oldManual)call(old,"setManual",true);Object fresh=make("sig.relation.AugmentationRelation");call(fresh,"setGrade",.7);call(fresh,"setManual",newManual);
   Object seq=make("sig.ui.UITaskList",new ArrayList<>());call(seq,"add",make("sig.ui.UnlinkTask",a[3],old));call(seq,"add",make("sig.ui.LinkTask",a[3],d,h1,fresh));
   List<Object> chords=List.of(c0,c1),voices=List.of(call(c0,"getVoice"),call(c1,"getVoice")),trace=new ArrayList<>(),pubs=new ArrayList<>(),steps=new ArrayList<>();watch(a,chords,voices,trace);
   Object index=call(a[1],"getInterIndex"),service=make("sig.ui.InterService",index,call(a[1],"getLocationService"));call(index,"setEntityService",service);Class<?> subType=Class.forName("org.bushe.swing.event.EventSubscriber");Object sub=Proxy.newProxyInstance(subType.getClassLoader(),new Class[]{subType},(p,m,v)->{if(m.getName().equals("hashCode"))return System.identityHashCode(p);if(m.getName().equals("equals"))return p==v[0];if(m.getName().equals("toString"))return "RelationTaskObserver";if(m.getName().equals("onEvent"))pubs.add(obj("source_is_dot",call(v[0],"getEntity")==d,"state",clocks(a,chords,voices)));return null;});call(service,"subscribeStrongly",Class.forName("org.audiveris.omr.ui.selection.EntityListEvent"),sub);
   Object before=clocks(a,chords,voices);List<Object> targets=new ArrayList<>();for(Object t:(List<?>)call(seq,"getTasks"))for(Object inter:List.of(call(t,"getSource"),call(t,"getTarget"))){Object st=call(a[2],"getStackAt",call(inter,"getCenter"));if(st!=null&&!targets.contains(st))targets.add(st);}
   for(String op:List.of("DO","UNDO","REDO")){pubs.clear();call(seq,op.equals("UNDO")?"performUndo":"performDo");javax.swing.SwingUtilities.invokeAndWait(()->{});Object after=clocks(a,chords,voices);call(rhythm,"impact",seq,Enum.valueOf((Class)c("sig.ui.UITask$OpKind"),op));steps.add(obj("operation",op.toLowerCase(),"old_manual",call(old,"isManual"),"new_manual",call(fresh,"isManual"),"old_grade",call(old,"getGrade"),"new_grade",call(fresh,"getGrade"),"after_task",after,"publications",new ArrayList<>(pubs),"after_rhythm_impact",clocks(a,chords,voices),"original_export_stream",exportStream(a[0])));
    if(op.equals("DO")) {
     call(fresh,"setGrade",.81);call(old,"setGrade",.31);int n=trace.size();pubs.clear();Object retainedLink=((List<?>)call(seq,"getTasks")).get(1);call(retainedLink,"performDo");javax.swing.SwingUtilities.invokeAndWait(()->{});
     Object repeated=clocks(a,chords,voices);call(rhythm,"impact",seq,Enum.valueOf((Class)c("sig.ui.UITask$OpKind"),"DO"));
     steps.add(obj("operation","repeat_link_do","callback_events",trace.size()-n,"publications",new ArrayList<>(pubs),"same_relation_registered",call(a[3],"containsEdge",fresh),"same_task_relation",call(retainedLink,"getRelation")==fresh,"live_grade",call(fresh,"getGrade"),"old_manual",call(old,"isManual"),"new_manual",call(fresh,"isManual"),"old_grade",call(old,"getGrade"),"new_grade",call(fresh,"getGrade"),"after_task",repeated,"after_rhythm_impact",clocks(a,chords,voices),"original_export_stream",exportStream(a[0])));
     Object retainedUnlink=((List<?>)call(seq,"getTasks")).get(0);String error=null;pubs.clear();int events=trace.size();
     try{call(retainedUnlink,"performDo");}catch(InvocationTargetException ex){error=ex.getCause().getClass().getSimpleName()+":"+ex.getCause().getMessage();}
     steps.add(obj("operation","repeat_unlink_do","original_exception",error,"callback_events",trace.size()-events,"publication_count",pubs.size(),"state",clocks(a,chords,voices)));
    }
   }
   List<Object> impacted=new ArrayList<>();for(Object step:c("step.OmrStep").getEnumConstants())if((Boolean)call(step,"isImpactedBy",fresh.getClass()))impacted.add(step.toString());
   cases.add(obj("old_manual",oldManual,"new_manual",newManual,"before",before,"tasks",List.of("UnlinkTask","LinkTask"),"impact_stack_count",targets.size(),"impact_stack_is_actual_first",targets.get(0)==a[4],"relation_impacted_steps",impacted,"steps",steps,"callback_trace",trace));
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd","scope","declared actual UnlinkTask then LinkTask; original UITaskList do/undo/redo, delivered source publication and actual RhythmsStep.impact; not automatic recognition/correction discovery","cases",cases));System.exit(0);
 }
}
