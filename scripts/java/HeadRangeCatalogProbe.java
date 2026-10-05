// SPDX-License-Identifier: AGPL-3.0-or-later
// Independent original 5.11 catalog and constructor eligibility. Never reads Rust outputs.
import org.audiveris.omr.sheet.Book;
import java.util.*;import java.nio.file.*;import java.awt.Point;import java.awt.geom.*;
public class HeadRangeCatalogProbe extends HeadRangePrefixProbe {
 boolean lastEligible; Object lastStaff;
 public HeadRangeCatalogProbe(Book b,SortedSet<Integer> ids){super(b,ids);}
 String shapes(Object s,String field)throws Exception{List<String> names=new ArrayList<>();for(Object v:(Collection<?>)get(s,field))names.add("\""+v+"\"");return "["+String.join(",",names)+"]";}
 String token(String json,String name){var m=java.util.regex.Pattern.compile("\""+name+"\":(\"[^\"]*\"|[^,}]+)").matcher(json);if(!m.find())throw new AssertionError(name);return m.group(1).replace("\"","");}
 String decision(String sh,double d,int x,int y)throws Exception{
  Object shape=Class.forName("org.audiveris.omr.glyph.Shape").getField(sh).get(null),loc=make("image.PixelDistance",x,y,d),b=get(activeScanner,"this$0"),staff=call(get(activeScanner,"line"),"getStaff");lastStaff=staff;
  boolean weak=(Boolean)call(activeScanner,"isWeakStemLessHead",shape,loc);
  Object a=Class.forName("org.audiveris.omr.image.Anchored$Anchor").getField("MIDDLE_LEFT").get(null);
  Object inter=call(b,"createInter",loc,a,shape,staff,((Number)get(activeScanner,"pitch")).doubleValue());
  double grade=((Number)call(make("sig.inter.HeadInter$Impacts",staticCall("image.Template","impactOf",d)),"getGrade")).doubleValue(),boost=(Double)staticCall("sheet.note.NoteHeadsBuilder","getStemLessBoost");
  boolean stemless=((Collection<?>)Class.forName("org.audiveris.omr.glyph.ShapeSet").getField("StemLessHeads").get(null)).contains(shape);
  double future=stemless?(Double)staticCall("sig.inter.AbstractInter","increaseGrade",grade,boost):grade;
  lastEligible=!weak&&inter!=null;
  return "{\"stemless\":"+stemless+",\"intrinsic_grade\":"+grade+",\"future_staff_grade\":"+future+",\"pending_stemless_boost\":"+(stemless?boost:0.)+",\"weak_stemless\":"+weak+",\"create_inter_eligible\":"+(inter!=null)+",\"range_eligible\":"+lastEligible+"}";
 }
 String match(Object table,Object catalog,int x,int y,int[]xo,int[]yo,String an,String sh)throws Exception{
  String raw=super.match(table,catalog,x,y,xo,yo,an,sh);if(raw.equals("null"))return raw;
  String gate=decision(token(raw,"shape"),Double.parseDouble(token(raw,"score")),Integer.parseInt(token(raw,"x")),Integer.parseInt(token(raw,"y")));
  return raw.substring(0,raw.length()-1)+",\"eligibility\":"+gate+"}";
 }
 String scan(Object b,Object adapter,Object second,Object table,Object catalog,int dir,int pitch,String identity)throws Exception{
  Object scanner=make("sheet.note.NoteHeadsBuilder$Scanner",b,adapter,second,dir,pitch,false);activeScanner=scanner;
  int left=Math.max((Integer)call(adapter,"getLeftAbscissa"),(Integer)call(call(adapter,"getStaff"),"getHeaderStop"));
  int right=(Integer)call(adapter,"getRightAbscissa")-(Integer)get(b,"minTemplateWidth"),half=(Integer)get(b,"templateHalf");
  int[]yo=(int[])get(scanner,"yOffsets");
  boolean[] relevant=right<left?new boolean[0]:(boolean[])call(scanner,"getRelevantBlackAbscissae",left,right);
  List<String> selected=new ArrayList<>(),visited=new ArrayList<>(),skipped=new ArrayList<>(),matches=new ArrayList<>(),rejected=new ArrayList<>();
  for(Object spot:(List<?>)call(b,"getGlyphsSlice",get(b,"systemSpots"),get(scanner,"competitorsArea")))selected.add(canonicalGlyph(spot));
  int width=(Integer)call(table,"getWidth"),height=(Integer)call(table,"getHeight");
  for(int x=left;x<=right;x++){
   int y=(Integer)call(scanner,"getTheoreticalOrdinate",x);String reason=null;
   if(x+half>=width)reason="right_limit";else if(y<0||y>=height)reason="ordinate_limit";else if((Integer)call(table,"getValue",x+half,y)/3>half)reason="no_foreground";
   if(reason!=null){skipped.add("["+x+","+y+",\""+reason+"\"]");x+=2*half-1;continue;}
   boolean black=relevant[x-left];visited.add("["+x+","+y+","+black+"]");
   for(Object shape:(Collection<?>)get(scanner,black?"scannerTemplateNotesAll":"scannerTemplateNotesHollow")){
    String m=match(table,catalog,x,y,new int[]{0},yo,"MIDDLE_LEFT",shape.toString());if(!m.equals("null")){if(lastEligible)matches.add(m);else rejected.add(m);}
   }
  }
  return "{"+identity+",\"dir\":"+dir+",\"pitch\":"+pitch+",\"scan_range\":["+left+","+right+"],\"template_half\":"+half+",\"y_offsets\":"+Arrays.toString(yo)+",\"selected_spots\":["+String.join(",",selected)+"],\"black_relevant_spans\":"+spans(relevant,left)+",\"visited\":["+String.join(",",visited)+"],\"skipped\":["+String.join(",",skipped)+"],\"all_shapes\":"+shapes(scanner,"scannerTemplateNotesAll")+",\"hollow_shapes\":"+shapes(scanner,"scannerTemplateNotesHollow")+",\"matches\":["+String.join(",",matches)+"],\"rejected\":["+String.join(",",rejected)+"]}";
 }

 void controls(Path output)throws Exception{
  Object scanner=activeScanner,b=get(scanner,"this$0"),catalog=get(b,"catalog"),anchor=Class.forName("org.audiveris.omr.image.Anchored$Anchor").getField("MIDDLE_LEFT").get(null);
  List<String> boundaries=new ArrayList<>(),images=new ArrayList<>();
  for(String shape:new String[]{"BREVE","WHOLE_NOTE","NOTEHEAD_VOID","NOTEHEAD_BLACK"})for(double d:new double[]{0.,Math.nextDown(.1875),.1875,Math.nextUp(.1875),Math.nextUp(Math.nextUp(.1875)),.4,Math.nextDown(.45),.45,Math.nextUp(.45),1.}){
   boundaries.add("{\"shape\":\""+shape+"\",\"score_bits\":"+Double.doubleToRawLongBits(d)+",\"eligibility\":"+decision(shape,d,40,45)+"}");
  }
  setField(scanner,"barAreas",new ArrayList<>());setField(scanner,"competitors",new ArrayList<>());
  Object family=Class.forName("org.audiveris.omr.ui.symbol.MusicFamily").getField("Bravura").get(null),font=staticCall("ui.symbol.MusicFont","getMusicFont",family,51);
  for(String shape:new String[]{"BREVE","WHOLE_NOTE","NOTEHEAD_VOID","NOTEHEAD_BLACK"}){
   Object sh=Class.forName("org.audiveris.omr.glyph.Shape").getField(shape).get(null),symbol=make("ui.symbol.TemplateSymbol",sh,family),template=call(catalog,"getTemplate",sh);
   java.awt.image.BufferedImage glyph=(java.awt.image.BufferedImage)call(symbol,"buildImage",font);
   Point2D offset=(Point2D)call(template,"getOffset",anchor);int px=40+(int)Math.rint(offset.getX()),py=35+(int)Math.rint(offset.getY());
   for(String variant:new String[]{"clean","erased","edge","empty"}){
    int w=128,h=96;byte[]pixels=new byte[w*h];Arrays.fill(pixels,(byte)255);int ox=variant.equals("edge")?-3:40,oy=variant.equals("edge")?-2:35;
    if(!variant.equals("empty"))for(int y=0;y<glyph.getHeight();y++)for(int x=0;x<glyph.getWidth();x++){int color=glyph.getRGB(x,y),xx=ox+x,yy=oy+y;boolean black=(color>>>24)>=140&&((color>>16)&255)<140;if(variant.equals("erased")&&x>glyph.getWidth()/2)black=false;if(xx>=0&&xx<w&&yy>=0&&yy<h)pixels[yy*w+xx]=(byte)(black?0:255);}
    Object bp=Class.forName("ij.process.ByteProcessor").getConstructor(int.class,int.class,byte[].class).newInstance(w,h,pixels);
    Object table=call(make("image.ChamferDistance$Short"),"computeToFore",bp);setField(b,"distances",table);
    String file=shape+"-"+variant+".png";java.awt.image.BufferedImage image=new java.awt.image.BufferedImage(w,h,java.awt.image.BufferedImage.TYPE_BYTE_GRAY);image.getRaster().setDataElements(0,0,w,h,pixels);javax.imageio.ImageIO.write(image,"PNG",output.resolveSibling(file).toFile());
    int x=variant.equals("edge")?px-43:px,y=variant.equals("edge")?py-37:py;
    for(boolean black:new boolean[]{false,true}){
     List<String> accepted=new ArrayList<>(),rejected=new ArrayList<>();for(String candidate:black?new String[]{"BREVE","WHOLE_NOTE","NOTEHEAD_VOID","NOTEHEAD_BLACK"}:new String[]{"BREVE","WHOLE_NOTE","NOTEHEAD_VOID"}){String m=match(table,catalog,x,y,new int[]{0},new int[]{0,-1,1},"MIDDLE_LEFT",candidate);if(!m.equals("null")){if(lastEligible)accepted.add(m);else rejected.add(m);}}
     images.add("{\"image\":\""+file+"\",\"binary_sha256\":\""+hash(pixels)+"\",\"rendered_shape\":\""+shape+"\",\"variant\":\""+variant+"\",\"pivot\":["+x+","+y+"],\"y_offsets\":[0,-1,1],\"black_relevant\":"+black+",\"matches\":["+String.join(",",accepted)+"],\"rejected\":["+String.join(",",rejected)+"]}");
    }
   }
  }
  Files.writeString(output.resolveSibling("controls.json"),"{\"native_outputs_used\":false,\"scope\":\"Original TemplateSymbol binary render, original ChamferDistance and Scanner.eval with explicitly empty competitor/bar lists; original stemless gate and constructor-only createInter, no SIG insertion. Score-only ULP controls call original gate/constructor.\",\"boundaries\":["+String.join(",",boundaries)+"],\"images\":["+String.join(",",images)+"]}\n");
 }
 public void process(){super.process();try{Path output=Path.of(System.getProperty("native.probe.output"));String saved=Files.readString(output);int i=saved.indexOf(",\"scope\":");saved=saved.substring(0,i)+",\"scope\":\"Original default Scanner all/hollow enum catalogs, range eval/hole conversion, isWeakStemLessHead and constructor-only createInter eligibility. Before aggregation, conflicts, glyph retrieval or SIG insertion.\",\"eligibility_parameters\":{\"stemless_boost\":"+staticCall("sheet.note.NoteHeadsBuilder","getStemLessBoost")+",\"min_contextual_grade\":"+Class.forName("org.audiveris.omr.glyph.Grades").getField("minContextualGrade").get(null)+",\"min_inter_grade\":"+staticCall("sig.inter.AbstractInter","getMinGrade")+"}}\n";Files.writeString(output,saved);controls(output);}catch(Exception e){throw new RuntimeException(e);}}
}
