// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;import java.util.*;import java.awt.Rectangle;import java.lang.reflect.*;import com.fasterxml.jackson.databind.ObjectMapper;
public class TimedDotSupportProbe extends StackReprocessProbe {
 static Object clocks(Object[] a,List<Object> chords,List<Object> initialVoices)throws Exception {
  List<Object> rows=new ArrayList<>();for(int i=0;i<chords.size();i++){Object ch=chords.get(i),v=call(ch,"getVoice"),slot=call(ch,"getSlot");rows.add(obj("bounds",rect(call(ch,"getBounds")),"dots",call(ch,"getDotsNumber"),"duration",RhythmResetProbe.rational(call(ch,"getDurationSansTuplet")),"end_time",RhythmResetProbe.rational(call(ch,"getEndTime")),"time_offset",RhythmResetProbe.rational(call(ch,"getTimeOffset")),"voice",v==null?null:call(v,"getId"),"slot_id",slot==null?null:call(slot,"getId"),"same_voice",v==initialVoices.get(i),"same_measure",call(ch,"getMeasure")==a[5],"same_part",call(ch,"getPart")==call(a[5],"getPart")));}
  return obj("chords",rows,"ownership",ownership(a[3],a[5],(List<Object>)a[6]),"rhythm",RhythmResetProbe.state(a[5]));
 }
 static void watch(Object[] a,List<Object> chords,List<Object> voices,List<Object> trace)throws Exception {
  Field f=Class.forName("org.jgrapht.graph.DefaultListenableGraph").getDeclaredField("graphListeners");f.setAccessible(true);List<Object> listeners=(List<Object>)f.get(a[3]);int[] depth={0};int found=0;
  for(int i=0;i<listeners.size();i++){Object delegate=listeners.get(i);if(!delegate.getClass().getSimpleName().equals("SigListener"))continue;found++;
   listeners.set(i,Proxy.newProxyInstance(delegate.getClass().getClassLoader(),new Class[]{Class.forName("org.jgrapht.event.GraphListener")},(p,m,args)->{
    if(!List.of("edgeAdded","edgeRemoved","vertexRemoved").contains(m.getName()))return m.invoke(delegate,args);
    Object e=call(args[0],m.getName().startsWith("edge")?"getEdge":"getVertex");int level=depth[0]++;
    trace.add(obj("event",m.getName(),"stage","enter","kind",e.getClass().getSimpleName(),"depth",level,"state",clocks(a,chords,voices)));
    try{return m.invoke(delegate,args);}finally{depth[0]--;trace.add(obj("event",m.getName(),"stage","leave","kind",e.getClass().getSimpleName(),"depth",level,"state",clocks(a,chords,voices)));}
   }));
  }if(found!=1)throw new IllegalStateException("original SigListener required");
 }
 static Object exportStream(Object book)throws Exception {
  Object score=((List<?>)call(book,"getScores")).get(0),xml=c("score.PartwiseBuilder").getMethod("build",c("score.Score")).invoke(null,score);
  int units=(Integer)call(call(score,"getFirstPage"),"simpleDurationOf",make("math.Rational",1,1));List<Object> stream=new ArrayList<>();int number=0;
  for(Object measure:(List<?>)call(((List<?>)call(xml,"getPart")).get(0),"getMeasure")){number++;Object onset=make("math.Rational",0,1),clock=onset;
   for(Object item:(List<?>)call(measure,"getNoteOrBackupOrForward")){String kind=item.getClass().getSimpleName();
    if(kind.equals("Note")){Object pitch=call(item,"getPitch"),d=make("math.Rational",((java.math.BigDecimal)call(item,"getDuration")).intValueExact(),units);boolean chord=call(item,"getChord")!=null;if(!chord){onset=clock;clock=call(clock,"plus",d);}stream.add(obj("measure_number",number,"step",call(pitch,"getStep").toString(),"octave",call(pitch,"getOctave"),"voice",Integer.parseInt((String)call(item,"getVoice")),"duration",d.toString(),"time_offset",onset.toString(),"type",call(call(item,"getType"),"getValue"),"dots",((List<?>)call(item,"getDot")).size(),"chord",chord));}
    else if(kind.equals("Backup")||kind.equals("Forward"))throw new IllegalStateException("single-voice declared control expected");
   }
  }return stream;
 }
 public static void main(String[] args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);List<Object> cases=new ArrayList<>();
  for(boolean manual:List.of(false,true)){
   Object[] a=load(args[0]);Object pageRhythm=make("sheet.rhythm.PageRhythm",call(a[2],"getPage"));call(pageRhythm,"reprocessStack",a[4]);Object d=((List<?>)a[6]).get(0),aug=null;
   for(Object e:(Collection<?>)call(a[3],"edgesOf",d))if(e.getClass().getSimpleName().equals("AugmentationRelation")){aug=e;break;}
   Object h0=call(a[3],"getEdgeTarget",aug),c0=call(h0,"getChord"),h1=null;
   List<Object> candidates=new ArrayList<>();for(Object ch:(Collection<?>)call(a[5],"getHeadChords"))if(ch!=c0&&call(ch,"getDurationSansDotOrTuplet").equals(call(c0,"getDurationSansDotOrTuplet")))candidates.add(ch);
   candidates.sort(Comparator.comparingInt(ch->{try{return ((Rectangle)call(ch,"getBounds")).x;}catch(Exception e){throw new RuntimeException(e);}}));
   if(candidates.isEmpty())throw new IllegalStateException("same-base owned chord required");Object c1=candidates.get(0);h1=((List<?>)call(c1,"getMembers")).get(0);
   if(manual)call(aug,"setManual",true);Object fresh=make("sig.relation.AugmentationRelation");call(fresh,"setGrade",.7);call(fresh,"setManual",true);
   List<Object> chords=List.of(c0,c1),voices=List.of(call(c0,"getVoice"),call(c1,"getVoice")),trace=new ArrayList<>(),steps=new ArrayList<>();Object before=clocks(a,chords,voices);
   Object editor=allocate(c("sig.ui.DefaultEditor"));field(editor,c("sheet.ui.ObjectEditor"),"object",d);field(editor,c("sheet.ui.ObjectEditor"),"system",a[2]);field(editor,c("sig.ui.DefaultEditor"),"originalBounds",call(d,"getBounds"));field(editor,c("sig.ui.DefaultEditor"),"latestBounds",call(d,"getBounds"));
   Object task=make("sig.ui.EditingTask",editor,List.of(make("sig.relation.Link",h1,fresh,true)),List.of(make("sig.relation.Link",h0,aug,true)));watch(a,chords,voices,trace);
   for(String op:List.of("do","undo","redo")){call(task,op.equals("undo")?"performUndo":"performDo");javax.swing.SwingUtilities.invokeAndWait(()->{});Object after=clocks(a,chords,voices);call(pageRhythm,"reprocessStack",a[4]);steps.add(obj("operation",op,"after_task",after,"after_reprocess",clocks(a,chords,voices),"original_export_stream",exportStream(a[0])));}
   cases.add(obj("original_support_manual",manual,"before",before,"steps",steps,"trace",trace));
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd","scope","declared same-base support transfer on actual saved owners after real PageRhythm.reprocessStack; original task callbacks then real retained reprocess; not automatic correction discovery","cases",cases));System.exit(0);
 }
}
