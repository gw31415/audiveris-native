// SPDX-License-Identifier: AGPL-3.0-or-later
// Original live seed glyphs, range aggregation, actual seed conflicts and range glyphs.
import org.audiveris.omr.sheet.Book;import java.util.*;import java.nio.file.*;import java.awt.Point;import java.awt.geom.*;
public class HeadGlyphProbe extends HeadCandidateFilterProbe {
 Set<Object> initialized=Collections.newSetFromMap(new IdentityHashMap<>());List<String> staffSeeds=new ArrayList<>();
 public HeadGlyphProbe(Book b,SortedSet<Integer> ids){super(b,ids);}
 String record(Object h)throws Exception {Object g=call(h,"getGlyph");Object tally=get(get(activeScanner,"this$0"),"tally");Object left=Class.forName("org.audiveris.omr.util.HorizontalSide").getField("LEFT").get(null),right=Class.forName("org.audiveris.omr.util.HorizontalSide").getField("RIGHT").get(null);return "{\"shape\":\""+call(h,"getShape")+"\",\"bounds\":"+rect(call(h,"getBounds"))+",\"grade\":"+call(h,"getGrade")+",\"pitch\":"+call(h,"getPitch")+",\"staff_id\":"+call(call(h,"getStaff"),"getId")+",\"glyph\":"+(g==null?"null":canonicalGlyph(g))+",\"left_dx\":"+call(tally,"getDx",h,left)+",\"right_dx\":"+call(tally,"getDx",h,right)+"}";}
 String scan(Object b,Object adapter,Object second,Object table,Object catalog,int dir,int pitch,String identity)throws Exception {
  Object staff=call(adapter,"getStaff");List<String> newSeeds=new ArrayList<>();
  if(initialized.add(staff)){
   Object key=Class.forName("org.audiveris.omr.sheet.Picture$SourceKey").getField("BINARY").get(null);setField(b,"image",call(call(sheet,"getPicture"),"getSource",key));Object group=Class.forName("org.audiveris.omr.glyph.GlyphGroup").getField("VERTICAL_SEED").get(null);List<?> seeds=(List<?>)call(get(b,"system"),"getGroupedGlyphs",group);setField(b,"systemSeeds",seeds);Collections.sort((List)seeds,(Comparator)Class.forName("org.audiveris.omr.glyph.Glyphs").getField("byOrdinate").get(null));
   List<?> hs=(List<?>)call(b,"processStaff",staff,true);activeScanner=make("sheet.note.NoteHeadsBuilder$Scanner",b,adapter,second,dir,pitch,false);
   for(Object h:hs)newSeeds.add(record(h));((List)get(b,"systemCompetitors")).addAll(hs);Collections.sort((List)get(b,"systemCompetitors"),(Comparator)Class.forName("org.audiveris.omr.sig.inter.Inters").getField("byOrdinate").get(null));
  }
  String raw=super.scan(b,adapter,second,table,catalog,dir,pitch,identity);
  List<?> aggregated=(List<?>)call(b,"aggregateMatches",new ArrayList<>(matches)),filtered=(List<?>)call(b,"filterSeedConflicts",aggregated,get(activeScanner,"competitors"));List<String>comps=new ArrayList<>(),glyphs=new ArrayList<>();
  for(Object h:(List<?>)get(activeScanner,"competitors"))if(h.getClass().getSimpleName().equals("HeadInter"))comps.add(record(h));
  for(Object h:filtered){Object t=call(catalog,"getTemplate",call(h,"getShape"));Object glyph=call(h,"retrieveGlyph",t,get(b,"image"));glyphs.add("{\"candidate_index\":"+positions.get(h)+",\"result\":"+(glyph==null?"null":record(h))+"}");}
  return raw.substring(0,raw.length()-1)+",\"glyph_retrieval\":{\"specific_interline\":"+call(staff,"getSpecificInterline")+",\"new_staff_seeds\":["+String.join(",",newSeeds)+"],\"seed_competitors\":["+String.join(",",comps)+"],\"filtered_indices\":"+indexes(filtered)+",\"range_glyphs\":["+String.join(",",glyphs)+"]}}";
 }

 void controls(Path output)throws Exception{
  super.controls(output);Object b=get(activeScanner,"this$0"),catalog=get(b,"catalog"),anchor=Class.forName("org.audiveris.omr.image.Anchored$Anchor").getField("MIDDLE_LEFT").get(null),family=Class.forName("org.audiveris.omr.ui.symbol.MusicFamily").getField("Bravura").get(null),font=staticCall("ui.symbol.MusicFont","getMusicFont",family,51);List<String> controls=new ArrayList<>();
  for(String shape:new String[]{"BREVE","WHOLE_NOTE","NOTEHEAD_VOID","NOTEHEAD_BLACK"}){
   Object sh=Class.forName("org.audiveris.omr.glyph.Shape").getField(shape).get(null),symbol=make("ui.symbol.TemplateSymbol",sh,family),template=call(catalog,"getTemplate",sh);java.awt.image.BufferedImage glyph=(java.awt.image.BufferedImage)call(symbol,"buildImage",font);Point2D offset=(Point2D)call(template,"getOffset",anchor);
   for(String variant:new String[]{"clean","erased","edge","empty"}){
    int w=128,h=96,ox=variant.equals("edge")?-3:40,oy=variant.equals("edge")?-2:35;byte[] pixels=new byte[w*h];Arrays.fill(pixels,(byte)255);
    if(!variant.equals("empty"))for(int y=0;y<glyph.getHeight();y++)for(int x=0;x<glyph.getWidth();x++){int color=glyph.getRGB(x,y),xx=ox+x,yy=oy+y;boolean black=(color>>>24)>=140&&((color>>16)&255)<140;if(variant.equals("erased")&&x>glyph.getWidth()/2)black=false;if(xx>=0&&xx<w&&yy>=0&&yy<h)pixels[yy*w+xx]=(byte)(black?0:255);}
    Object bp=Class.forName("ij.process.ByteProcessor").getConstructor(int.class,int.class,byte[].class).newInstance(w,h,pixels);java.awt.Rectangle slim=(java.awt.Rectangle)call(template,"getSlimBoundsAt",ox+(int)offset.getX(),oy+(int)offset.getY(),anchor);Object head=head(new int[]{slim.x,slim.y,slim.width,slim.height},.8,shape);String input="{\"shape\":\""+shape+"\",\"bounds\":"+rect(slim)+",\"grade\":0.8}";
    Object result=call(head,"retrieveGlyph",template,bp);String file=shape+"-"+variant+".png";java.awt.image.BufferedImage image=new java.awt.image.BufferedImage(w,h,java.awt.image.BufferedImage.TYPE_BYTE_GRAY);image.getRaster().setDataElements(0,0,w,h,pixels);javax.imageio.ImageIO.write(image,"PNG",output.resolveSibling(file).toFile());
    controls.add("{\"image\":\""+file+"\",\"binary_sha256\":\""+hash(pixels)+"\",\"input\":"+input+",\"glyph\":"+(result==null?"null":canonicalGlyph(result))+"}");
   }
  }
  List<String> pivots=new ArrayList<>();Object group=Class.forName("org.audiveris.omr.glyph.GlyphGroup").getField("VERTICAL_SEED").get(null);
  for(Object sys:(List<?>)call(sheet,"getSystems"))for(Object staff:(List<?>)call(sys,"getStaves"))for(Object line:(List<?>)call(staff,"getLines"))for(Object seed:(List<?>)call(sys,"getGroupedGlyphs",group)){
   java.awt.Rectangle box=(java.awt.Rectangle)call(seed,"getBounds");if(pivots.size()>=32&&box.x!=823)continue;
   Point2D center=(Point2D)call(seed,"getCenter2D"),a=(Point2D)call(seed,"getStartPoint",vertical),z=(Point2D)call(seed,"getStopPoint",vertical);int rough=(int)Math.rint(center.getX());Object adapter=make("sheet.note.NoteHeadsBuilder$StaffLineAdapter",b,staff,line);var method=adapter.getClass().getDeclaredMethod("yAt",int.class);method.setAccessible(true);int ordinate=(Integer)method.invoke(adapter,rough);int x=(int)Math.rint((Double)staticCall("math.LineUtil","xAtY",a,z,(double)ordinate));
   pivots.add("{\"line_bits\":["+Double.doubleToRawLongBits(a.getX())+","+Double.doubleToRawLongBits(a.getY())+","+Double.doubleToRawLongBits(z.getX())+","+Double.doubleToRawLongBits(z.getY())+"],\"ordinate_bits\":"+Double.doubleToRawLongBits((double)ordinate)+",\"x0\":"+x+",\"int_line_overload\":true}");
  }
  Files.writeString(output.resolveSibling("glyph-controls.json"),"{\"native_outputs_used\":false,\"scope\":\"Original TemplateSymbol binary renders and original HeadInter.retrieveGlyph; clean, erased, clipped edge and empty controls for all four shapes.\",\"controls\":["+String.join(",",controls)+"],\"pivot_controls\":["+String.join(",",pivots)+"]}\n");
 }
}
