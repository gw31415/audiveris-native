// SPDX-License-Identifier: AGPL-3.0-or-later
// Original controlled Scanner constructor observations, before pixel lookup.
import org.audiveris.omr.sheet.Book;import java.util.*;import java.nio.file.*;
public class HeadStaffCatalogProbe extends HeadRangeCatalogProbe {
 public HeadStaffCatalogProbe(Book b,SortedSet<Integer> ids){super(b,ids);}
 public static class TabStaff extends org.audiveris.omr.sheet.Staff {
  public TabStaff(int id,List lines){super(id,100.,1400.,11,lines);}
  @Override public boolean isTablature(){return true;}
 }
 String names(Collection<?> shapes){List<Object> sorted=new ArrayList<>(shapes);Collections.sort((List)sorted);List<String> rows=new ArrayList<>();for(Object s:sorted)rows.add("\""+s+"\"");return "["+String.join(",",rows)+"]";}
 void controls(Path output)throws Exception {
  Object sys=((List<?>)call(sheet,"getSystems")).get(0),base=((List<?>)call(sys,"getStaves")).get(0),scale=call(sheet,"getScale"),savedInterline=get(scale,"interlineScale"),switches=call(call(sheet,"getStub"),"getProcessingSwitches");
  String[] switchNames={"smallHeads","drumNotation","oneLineStaves","crossHeads"};Object[] keys=new Object[4],saved=new Object[4];for(int i=0;i<4;i++){keys[i]=Class.forName("org.audiveris.omr.sheet.ProcessingSwitch").getField(switchNames[i]).get(null);saved[i]=call(call(switches,"getParam",keys[i]),"getSpecific");}
  List<String> metadata=new ArrayList<>(),cases=new ArrayList<>(),outside=new ArrayList<>();
  for(String name:new String[]{"Heads","StemHeads","HollowHeads","HeadsOval","HeadsOvalSmall","HeadsCross","HeadsCrossHollow","HeadsDiamond","HeadsTriangle","HeadsCircle","QuarterHeads"})metadata.add("\""+name+"\":"+names((Collection<?>)Class.forName("org.audiveris.omr.glyph.ShapeSet").getField(name).get(null)));
  Object table=call(make("sheet.note.DistancesBuilder",sheet),"buildDistances");
  try {
   for(int main:new int[]{8,11,16})for(int specific:new int[]{7,11})for(int mode=0;mode<16;mode++){
    setField(scale,"interlineScale",make("sheet.Scale$InterlineScale",main,main,main));
    for(int i=0;i<4;i++)call(call(switches,"getParam",keys[i]),"setSpecific",(mode&(1<<i))!=0);
    Object b=make("sheet.note.NoteHeadsBuilder",sys,table,new ArrayList<>(),make("sheet.note.HeadSeedTally"),Boolean.FALSE,new TreeMap<>());
    setField(b,"systemBarAreas",new ArrayList<>());setField(b,"systemCompetitors",new ArrayList<>());
    for(int kind=0;kind<4;kind++){
     int count=kind==0?1:kind==3?4:5;
     List lines=kind==0?new ArrayList(List.of(((List<?>)call(base,"getLines")).get(2))):new ArrayList(((List<?>)call(base,"getLines")).subList(0,count));
     Object staff=make(kind==0?"sheet.OneLineStaff":"sheet.Staff",100+kind,100.,1400.,specific,lines);call(staff,"setSystem",sys);
     Object header=make("sheet.header.StaffHeader",100);if(kind==2||kind==3){Object shape=Class.forName("org.audiveris.omr.glyph.Shape").getField("PERCUSSION_CLEF").get(null);setField(header,"clef",make("sig.inter.ClefInter",shape,Double.valueOf(.8)));}else setField(header,"clef",get(call(base,"getHeader"),"clef"));put(staff,"header",header);
     List<String> scans=new ArrayList<>();Object previous=null;int pitch=-count;
     for(int li=0;li<count;li++){
      Object adapter=make("sheet.note.NoteHeadsBuilder$StaffLineAdapter",b,staff,lines.get(li));
      for(int dir:li==count-1?new int[]{-1,0,1}:new int[]{-1,0}){
       Object second=dir==-1?previous:null,scanner=make("sheet.note.NoteHeadsBuilder$Scanner",b,adapter,second,dir,pitch,false);
       scans.add("{\"line_index\":"+li+",\"dir\":"+dir+",\"pitch\":"+pitch+",\"second_line_index\":"+(second==null?"null":li-1)+",\"is_open\":"+get(scanner,"isOpen")+",\"y_offsets\":"+Arrays.toString((int[])get(scanner,"yOffsets"))+",\"all_shapes\":"+shapes(scanner,"scannerTemplateNotesAll")+",\"stem_shapes\":"+shapes(scanner,"scannerTemplateNotesStem")+",\"hollow_shapes\":"+shapes(scanner,"scannerTemplateNotesHollow")+"}");pitch++;
      }previous=adapter;
     }
     cases.add("{\"main_interline\":"+main+",\"specific_interline\":"+specific+",\"switch_mask\":"+mode+",\"line_count\":"+count+",\"drum\":"+call(staff,"isDrum")+",\"point_size\":"+call(staff,"getHeadPointSize")+",\"scanners\":["+String.join(",",scans)+"],\"ledger_allowed\":"+(count!=1)+"}");
     if(main==11&&specific==11&&mode==15&&(kind==0||kind==2||kind==3))for(int p=-12;p<=12;p++){
      Object adapter=make("sheet.note.NoteHeadsBuilder$StaffLineAdapter",b,staff,lines.get(0)),scanner=make("sheet.note.NoteHeadsBuilder$Scanner",b,adapter,null,p<0?-1:1,p,false);
      outside.add("{\"line_count\":"+count+",\"pitch\":"+p+",\"all_shapes\":"+shapes(scanner,"scannerTemplateNotesAll")+",\"stem_shapes\":"+shapes(scanner,"scannerTemplateNotesStem")+",\"hollow_shapes\":"+shapes(scanner,"scannerTemplateNotesHollow")+"}");
     }
    }
   }
  } finally {setField(scale,"interlineScale",savedInterline);for(int i=0;i<4;i++)call(call(switches,"getParam",keys[i]),"setSpecific",saved[i]);}
  // Explicit subclass control of original virtual tablature skip in buildHeads.
  List realStaffs=(List)call(sys,"getStaves"),savedStaffs=new ArrayList(realStaffs);List<String> tab=new ArrayList<>();
  try {for(int count:new int[]{4,6}){
   List lines=new ArrayList(((List<?>)call(base,"getLines")));while(lines.size()<count)lines.add(lines.get(lines.size()-1));lines=new ArrayList(lines.subList(0,count));Object staff=new TabStaff(200+count,lines);call(staff,"setSystem",sys);
   realStaffs.clear();realStaffs.add(staff);Object sig=call(sys,"getSig");int before=((Collection<?>)call(sig,"vertexSet")).size();Object b=make("sheet.note.NoteHeadsBuilder",sys,table,new ArrayList<>(),make("sheet.note.HeadSeedTally"),Boolean.FALSE,new TreeMap<>());call(b,"buildHeads");int after=((Collection<?>)call(sig,"vertexSet")).size();if(before!=after)throw new AssertionError("Tablature mutated SIG");tab.add("{\"line_count\":"+count+",\"tab_skipped\":true,\"SIG_vertex_delta\":"+(after-before)+"}");
  }}finally{realStaffs.clear();realStaffs.addAll(savedStaffs);}
  Files.writeString(output.resolveSibling("staff-catalog-controls.json"),"{\"native_outputs_used\":false,\"scope\":\"Controlled original Scanner constructors in source processStaff order; main/specific scale and 16 processing switch combinations. Original pitch-dependent drum catalogs and virtual tablature buildHeads skip. No physical special-staff full-page parity claim.\",\"shape_sets\":{"+String.join(",",metadata)+"},\"cases\":["+String.join(",",cases)+"],\"outside_pitch_controls\":["+String.join(",",outside)+"],\"tablature_controls\":["+String.join(",",tab)+"]}\n");
 }
}
