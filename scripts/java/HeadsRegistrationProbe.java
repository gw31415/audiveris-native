// SPDX-License-Identifier: AGPL-3.0-or-later
// Unmodified original complete NoteHeadsBuilder and HeadsStep epilog.
import java.awt.geom.Line2D;import org.audiveris.omr.sheet.Book;import java.util.*;import java.nio.file.*;
public class HeadsRegistrationProbe extends HeadGlyphProbe {
 public HeadsRegistrationProbe(Book b,SortedSet<Integer> ids){super(b,ids);}
 String vertex(Object h)throws Exception {return record(h).replaceFirst("\\{","{\"id\":"+call(h,"getId")+",\"abnormal\":"+call(h,"isAbnormal")+",\"removed\":"+call(h,"isRemoved")+",");}
 public void process(){try{
  sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");for(String step:new String[]{"sheet.grid.GridStep","sheet.header.HeadersStep","sheet.stem.StemSeedsStep","sheet.beam.BeamsStep","sheet.ledger.LedgersStep"})call(make(step),"doit",sheet);
  horizontal=Class.forName("org.audiveris.omr.run.Orientation").getField("HORIZONTAL").get(null);vertical=Class.forName("org.audiveris.omr.run.Orientation").getField("VERTICAL").get(null);
  Object step=make("sheet.note.HeadsStep"),ctx=call(step,"doProlog",sheet),dt=get(ctx,"distanceTable");List<String> systems=new ArrayList<>();
  for(Object sys:(List<?>)call(sheet,"getSystems")){
   Object sig=call(sys,"getSig");List<String> before=new ArrayList<>();for(Object v:(Collection<?>)call(sig,"vertexSet"))before.add(geometry(v));
   Object tally=((Map<?,?>)get(ctx,"tallies")).get(sys),b=make("sheet.note.NoteHeadsBuilder",sys,dt,((Map<?,?>)get(ctx,"sheetSpots")).get(sys),tally,Boolean.FALSE,new TreeMap<>());call(b,"buildHeads");
   Object staff=((List<?>)call(sys,"getStaves")).get(0),adapter=make("sheet.note.NoteHeadsBuilder$StaffLineAdapter",b,staff,((List<?>)call(staff,"getLines")).get(0));activeScanner=make("sheet.note.NoteHeadsBuilder$Scanner",b,adapter,null,0,-4,false);
   List<String> heads=new ArrayList<>(),others=new ArrayList<>(),relations=new ArrayList<>(),staffs=new ArrayList<>();
   for(Object v:(Collection<?>)call(sig,"vertexSet")){if(v.getClass().getSimpleName().equals("HeadInter"))heads.add(vertex(v));else {String row=geometry(v);others.add(row.substring(0,row.length()-1)+",\"contextual_grade\":"+call(v,"getContextualGrade")+"}");}}
   for(Object edge:(Collection<?>)call(sig,"edgeSet"))relations.add("{\"kind\":\""+edge.getClass().getSimpleName()+"\",\"source\":"+call(call(sig,"getEdgeSource",edge),"getId")+",\"target\":"+call(call(sig,"getEdgeTarget",edge),"getId")+"}");
   for(Object f:(List<?>)call(sys,"getStaves")){List<String> notes=new ArrayList<>();for(Object n:(Collection<?>)get(f,"notes"))notes.add(call(n,"getId").toString());staffs.add("{\"staff_id\":"+call(f,"getId")+",\"notes\":["+String.join(",",notes)+"]}");}
   systems.add("{\"system_id\":"+call(sys,"getId")+",\"before_inters\":["+String.join(",",before)+"],\"heads\":["+String.join(",",heads)+"],\"other_inters\":["+String.join(",",others)+"],\"relations\":["+String.join(",",relations)+"],\"staffs\":["+String.join(",",staffs)+"]}");
  }
  call(step,"doEpilog",sheet,ctx);Object scale=call(call(sheet,"getScale"),"getHeadSeedScale");List<String> dx=new ArrayList<>();for(String shape:new String[]{"NOTEHEAD_VOID","NOTEHEAD_BLACK"})for(String side:new String[]{"LEFT","RIGHT"}){Object sh=Class.forName("org.audiveris.omr.glyph.Shape").getField(shape).get(null),si=Class.forName("org.audiveris.omr.util.HorizontalSide").getField(side).get(null);dx.add("{\"shape\":\""+shape+"\",\"side\":\""+side+"\",\"dx\":"+call(scale,"getDx",sh,si)+"}");}
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"native_outputs_used\":false,\"systems\":["+String.join(",",systems)+"],\"head_seed_scale\":["+String.join(",",dx)+"],\"scope\":\"Unmodified complete original NoteHeadsBuilder.buildHeads with HeadsStep prolog and epilog; registration, duplicates, overlaps, staff notes, small beams and seed tally.\"}\n");
 nativeControls(((List<?>)call(sheet,"getSystems")).get(0),dt,Path.of(System.getProperty("native.probe.output")).resolveSibling("registration-controls.json"));
 extraControls(((List<?>)call(sheet,"getSystems")).get(0),dt,Path.of(System.getProperty("native.probe.output")).resolveSibling("epilog-controls.json"));
 }catch(Exception e){throw new RuntimeException(e);}}
 Object declared(Map<String,Object> r,Object staff,Object sig,Object tally)throws Exception{
  List<Integer> z=(List<Integer>)r.get("bounds");Object sh=Class.forName("org.audiveris.omr.glyph.Shape").getField((String)r.get("shape")).get(null);
  Object h=make("sig.inter.HeadInter",new java.awt.Rectangle(z.get(0),z.get(1),z.get(2),z.get(3)),sh,(Double)r.get("grade"),staff,(Double)r.get("pitch"));call(sig,"addVertex",h);
  for(String side:new String[]{"LEFT","RIGHT"}){Object dx=r.get(side.toLowerCase()+"_dx");if(dx!=null)call(tally,"putDx",h,Class.forName("org.audiveris.omr.util.HorizontalSide").getField(side).get(null),dx);}
  return h;
 }
 Map<String,Object> input(int x,int y,int w,int height,double grade,String shape,double pitch,Double left,Double right,int staff){Map<String,Object> r=new LinkedHashMap<>();r.put("bounds",List.of(x,y,w,height));r.put("grade",grade);r.put("shape",shape);r.put("pitch",pitch);r.put("left_dx",left);r.put("right_dx",right);r.put("staff",staff);return r;}
 void nativeControls(Object sys,Object table,Path out)throws Exception{
  Object sig=call(sys,"getSig");List<?> staves=(List<?>)call(sys,"getStaves");List<String> result=new ArrayList<>();List<List<Map<String,Object>>> cases=new ArrayList<>();List<String> names=new ArrayList<>();
  cases.add(new ArrayList<>());names.add("empty");
  cases.add(List.of(input(50,50,10,10,.5,"NOTEHEAD_BLACK",0.,null,null,0)));names.add("singleton");
  for(int mode=0;mode<15;mode++){
   int x=50,y=50,w=10,height=10,staff=0;double pitch=0.,grade=.5;String shape="NOTEHEAD_BLACK";Double left=null,right=null,l1=.5,r1=null;
   if(mode==0){l1=null;}
   if(mode==1){right=-.5;}
   if(mode==2){grade=.5-1e-5;right=-.5;}
   if(mode==3){grade=Math.nextUp(.5-1e-5);right=-.5;}
   if(mode==4){grade=.6;}
   if(mode==5){shape="NOTEHEAD_VOID";right=-.5;}
   if(mode==6){x=51;right=-.5;}
   if(mode==7){pitch=2.;x=51;}
   if(mode==8){x=58;}
   if(mode==9){x=52;y=59;}
   if(mode==10){x=52;y=59;staff=1;}
   if(mode==11){x=60;}
   if(mode==12){x=52;y=50;staff=1;}
   if(mode==13){x=55;y=55;w=20;height=20;}
   if(mode==14){grade=.399999;l1=.5;right=-.5;}
   cases.add(List.of(input(50,50,10,10,mode==13?.5:(mode==14?.399999:.5),"NOTEHEAD_BLACK",0.,l1,r1,0),input(x,y,w,height,grade,shape,pitch,left,right,staff)));names.add("pair-"+mode);
  }
  cases.add(List.of(input(50,50,20,20,.5,"NOTEHEAD_BLACK",0.,.5,null,0),input(60,60,20,20,.5,"NOTEHEAD_BLACK",0.,null,-.5,1)));names.add("area-quarter");
  cases.add(List.of(input(50,50,20,20,.5,"NOTEHEAD_BLACK",0.,.5,null,0),input(60,59,20,20,.5,"NOTEHEAD_BLACK",0.,null,-.5,1)));names.add("area-above-quarter");
  cases.add(List.of(input(50,50,10,10,.5,"NOTEHEAD_BLACK",0.,.5,null,0),input(50,50,10,10,.6,"NOTEHEAD_BLACK",0.,null,-.5,0),input(50,50,10,10,.7,"NOTEHEAD_BLACK",0.,.5,null,0)));names.add("three-left-removals");
  for(int k=0;k<cases.size();k++)for(boolean remove:new boolean[]{true,false}){
   Object tally=make("sheet.note.HeadSeedTally"),b=make("sheet.note.NoteHeadsBuilder",sys,table,new ArrayList<>(),tally,Boolean.FALSE,new TreeMap<>());setField(b,"systemBarAreas",new ArrayList<>());setField(b,"systemCompetitors",new ArrayList<>());Object f=staves.get(0),a=make("sheet.note.NoteHeadsBuilder$StaffLineAdapter",b,f,((List<?>)call(f,"getLines")).get(0));activeScanner=make("sheet.note.NoteHeadsBuilder$Scanner",b,a,null,0,-4,false);
   List<Object> all=new ArrayList<>();for(Map<String,Object> r:cases.get(k))all.add(declared(r,staves.get((Integer)r.get("staff")),sig,tally));List<Object> sorted=new ArrayList<>(all);Collections.sort(sorted,(Comparator)Class.forName("org.audiveris.omr.sig.inter.Inters").getField("byFullAbscissa").get(null));
   Class<?> ipp=Class.forName("org.audiveris.omr.sig.inter.InterPairPredicate");Object predicate=java.lang.reflect.Proxy.newProxyInstance(ipp.getClassLoader(),new Class[]{ipp},(p,m,args)->call(args[0],remove?"isSameAs":"overlaps",args[1]));int count=((Number)call(b,"purge",sorted,"control",predicate,remove)).intValue();call(tally,"purgeRemovedHeads");
   List<String> rows=new ArrayList<>(),edges=new ArrayList<>(),kept=new ArrayList<>();for(int i=0;i<all.size();i++){Object h=all.get(i);rows.add(record(h).replaceFirst("\\{","{\"index\":"+i+",\"removed\":"+call(h,"isRemoved")+","));if(!Boolean.TRUE.equals(call(h,"isRemoved")))kept.add(""+i);}
   for(Object edge:(Collection<?>)call(sig,"edgeSet")){if(!edge.getClass().getSimpleName().equals("Exclusion"))continue;int ai=all.indexOf(call(sig,"getEdgeSource",edge)),bi=all.indexOf(call(sig,"getEdgeTarget",edge));if(ai>=0&&bi>=0)edges.add("["+ai+","+bi+"]");}
   String input=encode(cases.get(k));result.add("{\"name\":\""+names.get(k)+"\",\"do_remove\":"+remove+",\"input\":"+input+",\"count\":"+count+",\"after\":["+String.join(",",rows)+"],\"kept_indices\":["+String.join(",",kept)+"],\"exclusions\":["+String.join(",",edges)+"]}");for(Object h:all)if(!Boolean.TRUE.equals(call(h,"isRemoved")))call(h,"remove");
  }
  Files.writeString(out,"{\"native_outputs_used\":false,\"scope\":\"Unmodified original purge, isSameAs, HeadInter.overlaps and HeadSeedTally; declared constructor-only heads with null glyphs.\",\"controls\":["+String.join(",",result)+"]}\n");
 }
 String encode(Object o){if(o==null)return "null";if(o instanceof String)return "\""+o+"\"";if(o instanceof Map m){List<String> a=new ArrayList<>();for(Object key:m.keySet())a.add(encode(key)+":"+encode(m.get(key)));return "{"+String.join(",",a)+"}";}if(o instanceof Collection c){List<String>a=new ArrayList<>();for(Object v:c)a.add(encode(v));return "["+String.join(",",a)+"]";}return o.toString();}
 Object controlBeam(Object sig,double length,double y2,double grade)throws Exception{Object beam=make("sig.inter.BeamInter",make("sig.inter.AbstractBeamInter$Impacts",1.,1.,1.,1.,1.,1.),new Line2D.Double(50,50,50+length,y2),8.);call(beam,"setGrade",grade);call(sig,"addVertex",beam);return beam;}
 String cgRecord(Object beam)throws Exception{String r=geometry(beam);return r.substring(0,r.length()-1)+",\"contextual_grade\":"+call(beam,"getContextualGrade")+",\"removed\":"+call(beam,"isRemoved")+"}";}
 void extraControls(Object sys,Object table,Path out)throws Exception{
  Object sig=call(sys,"getSig"),staff=((List<?>)call(sys,"getStaves")).get(0);List<String> context=new ArrayList<>(),small=new ArrayList<>(),scales=new ArrayList<>();
  for(int mode=0;mode<7;mode++){
   List<Object> beams=new ArrayList<>();beams.add(controlBeam(sig,10,50,.5));beams.add(controlBeam(sig,10,50,.6));beams.add(controlBeam(sig,10,50,.7));List<String> edges=new ArrayList<>();
   if(mode>0)for(int partner:new int[]{1,2}){if(mode==1&&partner==2)continue;Object e=make("sig.relation.BeamBeamRelation");double grade=mode==5?0.:mode==6?.5:1.;call(e,"setGrade",grade);call(sig,"addEdge",beams.get(0),beams.get(partner),e);edges.add("{\"kind\":\"BeamBeamRelation\",\"source\":1,\"target\":"+(partner+1)+",\"grade\":"+grade+"}");}
   if(mode==3){Object cause=Class.forName("org.audiveris.omr.sig.relation.Exclusion$ExclusionCause").getField("OVERLAP").get(null);call(sig,"insertExclusion",beams.get(1),beams.get(2),cause);edges.add("{\"kind\":\"Exclusion\",\"source\":2,\"target\":3}");}
   if(mode==4){Object e=make("sig.relation.BeamBeamRelation");call(e,"setGrade",.5);call(sig,"addEdge",beams.get(1),beams.get(0),e);edges.add("{\"kind\":\"BeamBeamRelation\",\"source\":2,\"target\":1,\"grade\":0.5}");}
   List<String>rows=new ArrayList<>();for(int i=0;i<beams.size();i++){Object b=beams.get(i);call(sig,"computeContextualGrade",b);rows.add(cgRecord(b).replaceFirst("\\{","{\"local_id\":"+(i+1)+","));}
   context.add("{\"mode\":"+mode+",\"beams\":["+String.join(",",rows)+"],\"relations\":["+String.join(",",edges)+"]}");for(Object b:beams)call(b,"remove");
  }
  for(int mode=0;mode<9;mode++){
   Object tally=make("sheet.note.HeadSeedTally"),b=make("sheet.note.NoteHeadsBuilder",sys,table,new ArrayList<>(),tally,Boolean.FALSE,new TreeMap<>());setField(b,"systemBarAreas",new ArrayList<>());setField(b,"systemCompetitors",new ArrayList<>());Object a=make("sheet.note.NoteHeadsBuilder$StaffLineAdapter",b,staff,((List<?>)call(staff,"getLines")).get(0));activeScanner=make("sheet.note.NoteHeadsBuilder$Scanner",b,a,null,0,-4,false);Object beam=controlBeam(sig,mode==4?28:10,mode==6?60:50,.5);List<Object>heads=new ArrayList<>();List<Map<String,Object>> input=new ArrayList<>();
   if(mode!=0){double grade=mode==1?.6:mode==2?.5:.4;int x=mode==5?60:52,y=mode==3?54:48;if(mode==6){x=58;y=46;}input.add(input(x,y,4,4,grade,"NOTEHEAD_BLACK",0.,null,null,0));if(mode==7)input.add(input(53,49,4,4,.6,"NOTEHEAD_VOID",0.,null,null,0));if(mode==8)input.add(input(53,49,4,4,.5,"WHOLE_NOTE",0.,null,null,0));}
   for(Map<String,Object> r:input)heads.add(declared(r,staff,sig,tally));String before=geometry(beam);call(b,"purgeSmallBeams");List<String>rows=new ArrayList<>();for(Object h:heads)rows.add(record(h).replaceFirst("\\{","{\"removed\":"+call(h,"isRemoved")+","));small.add("{\"mode\":"+mode+",\"beam_input\":"+before+",\"head_inputs\":"+encode(input)+",\"beam_after\":"+cgRecord(beam)+",\"heads_after\":["+String.join(",",rows)+"]}");for(Object h:heads)if(!Boolean.TRUE.equals(call(h,"isRemoved")))call(h,"remove");if(!Boolean.TRUE.equals(call(beam,"isRemoved")))call(beam,"remove");
  }
  for(int n:new int[]{0,9,10,11}){
   Object tally=make("sheet.note.HeadSeedTally");List<Object> heads=new ArrayList<>();List<Map<String,Object>> input=new ArrayList<>();for(int i=0;i<n;i++){Map<String,Object> r=input(50+i*20,50,10,10,.5,i%2==0?"NOTEHEAD_BLACK":"NOTEHEAD_VOID",0.,i+.5,-i-.5,0); // each shape quorum is controlled independently below
    r.put("shape","NOTEHEAD_BLACK");input.add(r);Object h=declared(r,staff,sig,tally);heads.add(h);if(n==11&&i==10){call(h,"remove");r.put("removed",true);}}
   staticCall("sheet.note.HeadSeedTally","analyze",sheet,List.of(tally));Object scale=call(call(sheet,"getScale"),"getHeadSeedScale");List<String> dx=new ArrayList<>();for(String sh:new String[]{"NOTEHEAD_VOID","NOTEHEAD_BLACK"})for(String side:new String[]{"LEFT","RIGHT"})dx.add("{\"shape\":\""+sh+"\",\"side\":\""+side+"\",\"dx\":"+call(scale,"getDx",Class.forName("org.audiveris.omr.glyph.Shape").getField(sh).get(null),Class.forName("org.audiveris.omr.util.HorizontalSide").getField(side).get(null))+"}");scales.add("{\"n\":"+n+",\"input\":"+encode(input)+",\"head_seed_scale\":["+String.join(",",dx)+"]}");for(Object h:heads)if(!Boolean.TRUE.equals(call(h,"isRemoved")))call(h,"remove");
  }
  Files.writeString(out,"{\"native_outputs_used\":false,\"context\":["+String.join(",",context)+"],\"small_beams\":["+String.join(",",small)+"],\"seed_scale\":["+String.join(",",scales)+"]}\n");
 }
}
