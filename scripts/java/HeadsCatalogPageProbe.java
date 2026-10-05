// SPDX-License-Identifier: AGPL-3.0-or-later
// Unmodified original complete NoteHeadsBuilder and HeadsStep epilog.
import java.awt.geom.Line2D;import org.audiveris.omr.sheet.Book;import java.util.*;import java.nio.file.*;
public class HeadsCatalogPageProbe extends HeadGlyphProbe {
 public HeadsCatalogPageProbe(Book b,SortedSet<Integer> ids){super(b,ids);}
 String vertex(Object h)throws Exception {return record(h).replaceFirst("\\{","{\"id\":"+call(h,"getId")+",\"abnormal\":"+call(h,"isAbnormal")+",\"removed\":"+call(h,"isRemoved")+",");}
 public void process(){try{
  sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");for(String step:new String[]{"sheet.grid.GridStep","sheet.header.HeadersStep","sheet.stem.StemSeedsStep","sheet.beam.BeamsStep","sheet.ledger.LedgersStep"})call(make(step),"doit",sheet);
  horizontal=Class.forName("org.audiveris.omr.run.Orientation").getField("HORIZONTAL").get(null);vertical=Class.forName("org.audiveris.omr.run.Orientation").getField("VERTICAL").get(null);
  String familyName=System.getProperty("native.head.family","Bravura");int point=Integer.parseInt(System.getProperty("native.head.point","51"));
  Object stub=call(sheet,"getStub"),family=Class.forName("org.audiveris.omr.ui.symbol.MusicFamily").getField(familyName).get(null);call(call(stub,"getMusicFamilyParam"),"setSpecific",family);call(call(sheet,"getScale"),"setMusicFontScale",make("sheet.Scale$MusicFontScale",familyName,point));
  if(Boolean.getBoolean("native.head.cue"))setField(((List<?>)call(((List<?>)call(sheet,"getSystems")).get(0),"getStaves")).get(0),"specificInterline",7);
  Object step=make("sheet.note.HeadsStep"),ctx=call(step,"doProlog",sheet),dt=get(ctx,"distanceTable");List<String> systems=new ArrayList<>();
  for(Object sys:(List<?>)call(sheet,"getSystems")){
   Object sig=call(sys,"getSig");List<String> before=new ArrayList<>();for(Object v:(Collection<?>)call(sig,"vertexSet"))before.add(geometry(v));
   Object tally=((Map<?,?>)get(ctx,"tallies")).get(sys),b=make("sheet.note.NoteHeadsBuilder",sys,dt,((Map<?,?>)get(ctx,"sheetSpots")).get(sys),tally,Boolean.FALSE,new TreeMap<>());call(b,"buildHeads");
   Object staff=((List<?>)call(sys,"getStaves")).get(0),adapter=make("sheet.note.NoteHeadsBuilder$StaffLineAdapter",b,staff,((List<?>)call(staff,"getLines")).get(0));activeScanner=make("sheet.note.NoteHeadsBuilder$Scanner",b,adapter,null,0,-4,false);
   List<String> heads=new ArrayList<>(),others=new ArrayList<>(),relations=new ArrayList<>(),staffs=new ArrayList<>();
   for(Object v:(Collection<?>)call(sig,"vertexSet")){if(v.getClass().getSimpleName().equals("HeadInter"))heads.add(vertex(v));else {String row=geometry(v);others.add(row.substring(0,row.length()-1)+",\"contextual_grade\":"+call(v,"getContextualGrade")+"}");}}
   for(Object edge:(Collection<?>)call(sig,"edgeSet"))relations.add("{\"kind\":\""+edge.getClass().getSimpleName()+"\",\"source\":"+call(call(sig,"getEdgeSource",edge),"getId")+",\"target\":"+call(call(sig,"getEdgeTarget",edge),"getId")+"}");
   for(Object f:(List<?>)call(sys,"getStaves")){List<String> notes=new ArrayList<>();for(Object n:get(f,"notes")==null?List.of():(Collection<?>)get(f,"notes"))notes.add(call(n,"getId").toString());staffs.add("{\"staff_id\":"+call(f,"getId")+",\"notes\":["+String.join(",",notes)+"]}");}
   systems.add("{\"system_id\":"+call(sys,"getId")+",\"before_inters\":["+String.join(",",before)+"],\"heads\":["+String.join(",",heads)+"],\"other_inters\":["+String.join(",",others)+"],\"relations\":["+String.join(",",relations)+"],\"staffs\":["+String.join(",",staffs)+"]}");
  }
  call(step,"doEpilog",sheet,ctx);Object scale=call(call(sheet,"getScale"),"getHeadSeedScale");List<String> dx=new ArrayList<>();for(String shape:new String[]{"NOTEHEAD_VOID","NOTEHEAD_BLACK"})for(String side:new String[]{"LEFT","RIGHT"}){Object sh=Class.forName("org.audiveris.omr.glyph.Shape").getField(shape).get(null),si=Class.forName("org.audiveris.omr.util.HorizontalSide").getField(side).get(null);dx.add("{\"shape\":\""+shape+"\",\"side\":\""+side+"\",\"dx\":"+(scale==null?"null":call(scale,"getDx",sh,si))+"}");}
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"native_outputs_used\":false,\"systems\":["+String.join(",",systems)+"],\"head_seed_scale\":["+String.join(",",dx)+"],\"scope\":\"Unmodified complete original NoteHeadsBuilder.buildHeads with HeadsStep prolog and epilog on the same Laura raster after original Bravura GRID/HEADERS/BEAMS/LEDGERS; explicitly configured HEADS music family and stored point size. Catalog propagation control, not a page rendered in another font.\"}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
