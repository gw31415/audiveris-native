// SPDX-License-Identifier: AGPL-3.0-or-later
// Independent original Scanner.eval seed windows on official rendered symbols.
// No native outputs; matching before physical glyph retrieval or SIG insertion.
import org.audiveris.omr.sheet.Book;
import java.util.*;import java.nio.file.*;import java.awt.geom.*;
public class HeadSeedCoverageProbe extends HeadVetoProbe {
 public HeadSeedCoverageProbe(Book b,SortedSet<Integer> ids){super(b,ids);}
 public void process(){super.process();try{controls(Path.of(System.getProperty("native.probe.output")));}catch(Exception e){throw new RuntimeException(e);}}
 void controls(Path output)throws Exception {
  Object scanner=activeScanner,b=get(scanner,"this$0"),factory=staticCall("image.TemplateFactory","getInstance");
  setField(scanner,"competitors",new ArrayList<>());
  String[] shapes={"NOTEHEAD_VOID","NOTEHEAD_BLACK","NOTEHEAD_BLACK_SMALL","NOTEHEAD_CROSS","NOTEHEAD_DIAMOND_FILLED","NOTEHEAD_TRIANGLE_DOWN_FILLED","NOTEHEAD_CIRCLE_X"};
  List<String> images=new ArrayList<>();
  for(String familyName:new String[]{"Bravura","Leland","FinaleJazz"})for(int size:new int[]{32,40,51,64}){
   Object family=Class.forName("org.audiveris.omr.ui.symbol.MusicFamily").getField(familyName).get(null);
   Object font=staticCall("ui.symbol.MusicFont","getMusicFont",family,size),catalog=call(factory,"getCatalog",family,size);setField(b,"catalog",catalog);
   for(String shape:shapes){
    Object sh=Class.forName("org.audiveris.omr.glyph.Shape").getField(shape).get(null),symbol=make("ui.symbol.TemplateSymbol",sh,family),template=call(catalog,"getTemplate",sh);
    java.awt.image.BufferedImage glyph=(java.awt.image.BufferedImage)call(symbol,"buildImage",font);
    for(String variant:new String[]{"clean","erased","edge","empty"}){
     int w=128,h=96,ox=variant.equals("edge")?-3:40,oy=variant.equals("edge")?-2:35;byte[] pixels=new byte[w*h];Arrays.fill(pixels,(byte)255);
     if(!variant.equals("empty"))for(int y=0;y<glyph.getHeight();y++)for(int x=0;x<glyph.getWidth();x++){
      int color=glyph.getRGB(x,y),xx=ox+x,yy=oy+y;boolean black=(color>>>24)>=140&&((color>>16)&255)<140;
      if(variant.equals("erased")&&x>glyph.getWidth()/2)black=false;
      if(xx>=0&&xx<w&&yy>=0&&yy<h)pixels[yy*w+xx]=(byte)(black?0:255);
     }
     Object bp=Class.forName("ij.process.ByteProcessor").getConstructor(int.class,int.class,byte[].class).newInstance(w,h,pixels);
     Object table=call(make("image.ChamferDistance$Short"),"computeToFore",bp);setField(b,"distances",table);
     String file=familyName+"-"+size+"-"+shape+"-"+variant+".png";
     java.awt.image.BufferedImage image=new java.awt.image.BufferedImage(w,h,java.awt.image.BufferedImage.TYPE_BYTE_GRAY);image.getRaster().setDataElements(0,0,w,h,pixels);javax.imageio.ImageIO.write(image,"PNG",output.resolveSibling(file).toFile());
     List<String> windows=new ArrayList<>();
     for(String an:new String[]{"LEFT_STEM","RIGHT_STEM"}){
      Object anchor=Class.forName("org.audiveris.omr.image.Anchored$Anchor").getField(an).get(null);Point2D offset=(Point2D)call(template,"getOffset",anchor);
      int px=ox+(int)Math.rint(offset.getX()),py=oy+(int)Math.rint(offset.getY());java.awt.Rectangle slim=(java.awt.Rectangle)call(template,"getSlimBoundsAt",px,py,anchor);
      for(String mode:new String[]{"none","central","shifted"}){
       java.awt.Rectangle r=mode.equals("none")?null:mode.equals("central")?new java.awt.Rectangle(-1000,-1000,2000,2000):new java.awt.Rectangle(slim.x+slim.width,slim.y-5,1,slim.height+10);
       setField(scanner,"barAreas",r==null?new ArrayList<>():new ArrayList<>(List.of(new Area(r))));
       int evaluations=evalCount,vetoes=barVetoCount,central=centralVetoCount,noncentral=noncentralVetoCount;
       List<String> matches=new ArrayList<>();for(String candidate:shapes){String m=match(table,catalog,px,py,new int[]{0,1,-1,2,-2},new int[]{0,-1,1},an,candidate);if(!m.equals("null"))matches.add(m);}
       windows.add("{\"anchor\":\""+an+"\",\"pivot\":["+px+","+py+"],\"x_offsets\":[0,1,-1,2,-2],\"y_offsets\":[0,-1,1],\"veto_mode\":\""+mode+"\",\"veto_rectangle\":"+(r==null?"null":rect(r))+",\"original_eval_count\":"+(evalCount-evaluations)+",\"original_bar_veto_count\":"+(barVetoCount-vetoes)+",\"original_central_veto_count\":"+(centralVetoCount-central)+",\"original_noncentral_veto_count\":"+(noncentralVetoCount-noncentral)+",\"matches\":["+String.join(",",matches)+"]}");
      }
     }
     String folder=(shape.equals("NOTEHEAD_BLACK")||shape.equals("NOTEHEAD_VOID"))?"official-head-font-coverage":"official-head-drum-coverage";
     images.add("{\"family\":\""+familyName+"\",\"point_size\":"+size+",\"rendered_shape\":\""+shape+"\",\"variant\":\""+variant+"\",\"image\":\""+folder+"/"+file+"\",\"binary_sha256\":\""+hash(pixels)+"\",\"windows\":["+String.join(",",windows)+"]}");
    }
   }
  }
  List<String> names=new ArrayList<>();for(String s:shapes)names.add("\""+s+"\"");
  Files.writeString(output.resolveSibling("seed-controls.json"),"{\"native_outputs_used\":false,\"source_shapes\":["+String.join(",",names)+"],\"scope\":\"Exact declared seven-shape seed windows using original Scanner.eval, strict location ties, central/noncentral original frozen-bar veto and BLACK-only hole reassignment. Three official fonts and four point sizes. No physical seed pivot selection, glyph retrieval, staff mapping or full page catalog propagation claimed. Existing official rendered PNGs are independently reproduced and hash checked.\",\"images\":["+String.join(",",images)+"]}\n");
 }
}
