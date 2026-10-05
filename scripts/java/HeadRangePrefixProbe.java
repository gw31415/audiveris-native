// SPDX-License-Identifier: AGPL-3.0-or-later
// Original live HEAD_SPOTS / Scanner lookupRange prefix. No Rust outputs read.
// Freeze before createInter/aggregation/filterSeedConflicts/retrieveGlyph.
import org.audiveris.omr.sheet.Book;
import java.util.*;import java.nio.file.*;import java.awt.Point;import java.awt.geom.*;import java.security.*;
public class HeadRangePrefixProbe extends HeadVetoProbe {
 public HeadRangePrefixProbe(Book b,SortedSet<Integer> ids){super(b,ids);}
 String spans(boolean[] flags,int left){List<String> out=new ArrayList<>();for(int i=0;i<flags.length;){if(!flags[i]){i++;continue;}int start=i;while(i<flags.length&&flags[i])i++;out.add("["+(left+start)+","+(left+i-1)+"]");}return "["+String.join(",",out)+"]";}
 String scan(Object b,Object adapter,Object second,Object table,Object catalog,int dir,int pitch,String identity)throws Exception{
  Object scanner=make("sheet.note.NoteHeadsBuilder$Scanner",b,adapter,second,dir,pitch,false);activeScanner=scanner;
  int left=Math.max((Integer)call(adapter,"getLeftAbscissa"),(Integer)call(call(adapter,"getStaff"),"getHeaderStop"));
  int right=(Integer)call(adapter,"getRightAbscissa")-(Integer)get(b,"minTemplateWidth"),half=(Integer)get(b,"templateHalf");
  int[]yo=(int[])get(scanner,"yOffsets");
  boolean[] relevant=right<left?new boolean[0]:(boolean[])call(scanner,"getRelevantBlackAbscissae",left,right);
  List<String> selected=new ArrayList<>(),visited=new ArrayList<>(),skipped=new ArrayList<>(),matches=new ArrayList<>();
  for(Object spot:(List<?>)call(b,"getGlyphsSlice",get(b,"systemSpots"),get(scanner,"competitorsArea")))selected.add(canonicalGlyph(spot));
  int width=(Integer)call(table,"getWidth"),height=(Integer)call(table,"getHeight");
  for(int x=left;x<=right;x++){
   int y=(Integer)call(scanner,"getTheoreticalOrdinate",x);String reason=null;
   if(x+half>=width)reason="right_limit";else if(y<0||y>=height)reason="ordinate_limit";else if((Integer)call(table,"getValue",x+half,y)/3>half)reason="no_foreground";
   if(reason!=null){skipped.add("["+x+","+y+",\""+reason+"\"]");x+=2*half-1;continue;}
   boolean black=relevant[x-left];visited.add("["+x+","+y+","+black+"]");
   for(String shape:black?new String[]{"NOTEHEAD_BLACK","NOTEHEAD_VOID"}:new String[]{"NOTEHEAD_VOID"}){
    String m=match(table,catalog,x,y,new int[]{0},yo,"MIDDLE_LEFT",shape);if(!m.equals("null"))matches.add(m);
   }
  }
  return "{"+identity+",\"dir\":"+dir+",\"pitch\":"+pitch+",\"scan_range\":["+left+","+right+"],\"template_half\":"+half+",\"y_offsets\":"+Arrays.toString(yo)+",\"selected_spots\":["+String.join(",",selected)+"],\"black_relevant_spans\":"+spans(relevant,left)+",\"visited\":["+String.join(",",visited)+"],\"skipped\":["+String.join(",",skipped)+"],\"matches\":["+String.join(",",matches)+"]}";
 }
 public void process(){try{
  sheet=call(((java.util.List<?>)call(book,"getStubs")).get(0),"getSheet");
  for(String step:new String[]{"sheet.grid.GridStep","sheet.header.HeadersStep","sheet.stem.StemSeedsStep","sheet.beam.BeamsStep","sheet.ledger.LedgersStep"})call(make(step),"doit",sheet);
  horizontal=Class.forName("org.audiveris.omr.run.Orientation").getField("HORIZONTAL").get(null);vertical=Class.forName("org.audiveris.omr.run.Orientation").getField("VERTICAL").get(null);
  Object table=call(make("sheet.note.DistancesBuilder",sheet),"buildDistances");int w=(Integer)call(table,"getWidth"),h=(Integer)call(table,"getHeight");
  MessageDigest dh=MessageDigest.getInstance("SHA-256"),mh=MessageDigest.getInstance("SHA-256");for(int i=0;i<w*h;i++){int v=(Integer)call(table,"getValue",i);dh.update((byte)(v>>>8));dh.update((byte)v);mh.update(v==-1?(byte)0:(byte)255);}
  Map<?,?> spotMap=(Map<?,?>)call(make("sheet.note.HeadSpotsBuilder",sheet),"getSpots");
  Object key=Class.forName("org.audiveris.omr.sheet.Picture$TableKey").getField("HEAD_SPOTS").get(null);
  Object headRuns=call(call(sheet,"getPicture"),"getVerticalTable",key),buffer=call(headRuns,"getBuffer");
  byte[]pixels=(byte[])call(buffer,"getPixels");
  javax.imageio.ImageIO.write((java.awt.image.BufferedImage)call(buffer,"getBufferedImage"),"PNG",Path.of(System.getProperty("native.probe.output")).resolveSibling("head-spots.png").toFile());
  Object family=Class.forName("org.audiveris.omr.ui.symbol.MusicFamily").getField("Bravura").get(null),factory=staticCall("image.TemplateFactory","getInstance");
  List<String> systems=new ArrayList<>();int references=0;
  for(Object sys:(List<?>)call(sheet,"getSystems")){
   List<?> spots=(List<?>)spotMap.get(sys);List<String> spotRecords=new ArrayList<>(),staffRows=new ArrayList<>();
   references+=spots.size();for(Object spot:spots){Point c=(Point)call(spot,"getCentroid");spotRecords.add("{\"glyph\":"+canonicalGlyph(spot)+",\"centroid\":["+c.x+","+c.y+"],\"groups\":[\"HEAD_SPOT\"]}");}
   Object b=make("sheet.note.NoteHeadsBuilder",sys,table,new ArrayList<>(spots),make("sheet.note.HeadSeedTally"),Boolean.FALSE,new TreeMap<>());
   setField(b,"systemBarAreas",call(b,"getSystemBarAreas"));setField(b,"systemCompetitors",call(b,"getSystemCompetitors"));
   Collections.sort((List)get(b,"systemSpots"),(Comparator)Class.forName("org.audiveris.omr.glyph.Glyphs").getField("byOrdinate").get(null));
   for(Object staff:(List<?>)call(sys,"getStaves")){
    int size=(Integer)call(staff,"getHeadPointSize");Object catalog=call(factory,"getCatalog",family,size);setField(b,"catalog",catalog);
    List<?> lines=(List<?>)call(staff,"getLines");List<String> scans=new ArrayList<>(),ledgerScans=new ArrayList<>();Object previous=null;int pitch=-5;
    for(int li=0;li<lines.size();li++){Object adapter=make("sheet.note.NoteHeadsBuilder$StaffLineAdapter",b,staff,lines.get(li));for(int dir:li==4?new int[]{-1,0,1}:new int[]{-1,0}){
      scans.add(scan(b,adapter,dir==-1?previous:null,table,catalog,dir,pitch++,"\"line_index\":"+li));}previous=adapter;}
    Object part=call(staff,"getPart");boolean merged=(Boolean)call(part,"isMerged");
    for(int sign:new int[]{-1,1}){boolean further=!(merged&&(sign>0&&call(part,"getFirstStaff")==staff||sign<0&&call(part,"getLastStaff")==staff));
     for(int index=sign;;index+=sign){List<?> ls=(List<?>)call(staff,"getLedgers",index);if(ls==null||ls.isEmpty())break;int p=sign*(4+2*Math.abs(index));
      for(int n=0;n<ls.size();n++){Object a=make("sheet.note.NoteHeadsBuilder$LedgerAdapter",b,staff,"",call(ls.get(n),"getGlyph"));String identity="\"ledger_index\":"+index+",\"ledger_order\":"+n;
       ledgerScans.add(scan(b,a,null,table,catalog,0,p,identity));if(further)ledgerScans.add(scan(b,a,null,table,catalog,sign,p+sign,identity));}}}
    staffRows.add("{\"staff_id\":"+call(staff,"getId")+",\"point_size\":"+size+",\"header_stop\":"+call(staff,"getHeaderStop")+",\"scanners\":["+String.join(",",scans)+"],\"ledger_scanners\":["+String.join(",",ledgerScans)+"]}");
   }
   systems.add("{\"system_id\":"+call(sys,"getId")+",\"spots\":["+String.join(",",spotRecords)+"],\"staffs\":["+String.join(",",staffRows)+"]}");
  }
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"native_outputs_used\":false,\"head_spots_sha256\":\""+hash(pixels)+"\",\"system_spot_references\":"+references+",\"distance_sha256_be_i16\":\""+HexFormat.of().formatHex(dh.digest())+"\",\"mask_sha256\":\""+HexFormat.of().formatHex(mh.digest())+"\",\"systems\":["+String.join(",",systems)+"],\"eval_counts\":{\"eval\":"+evalCount+",\"bar_veto\":"+barVetoCount+",\"competitor_veto\":"+competitorVetoCount+",\"central_veto\":"+centralVetoCount+",\"noncentral_veto\":"+noncentralVetoCount+"},\"scope\":\"Original HeadSpotsBuilder.getSpots, Scanner range limits, spot slices/relevance, safety skips, ordinates and black/void MIDDLE_LEFT eval/hole reassignment; before candidate aggregation, seed conflicts, glyph retrieval or registration. Other catalog shapes excluded explicitly.\"}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
