// SPDX-License-Identifier: AGPL-3.0-or-later
// Original length-based support on explicitly selected physical stems.
import org.audiveris.omr.sheet.Book;import java.util.*;import java.nio.file.*;import java.awt.Rectangle;import java.awt.geom.*;
public class HeadStemSupportProbe extends HeadStemConnectionProbe {
 public HeadStemSupportProbe(Book b,SortedSet<Integer> ids){super(b,ids);}
 @Override void controls(Path out)throws Exception {
  Object sig=call(call(lastStaff,"getSystem"),"getSig"),scale=call(sheet,"getScale"),factory=staticCall("image.TemplateFactory","getInstance"),savedScale=call(scale,"getHeadSeedScale");List<String> batches=new ArrayList<>();
  String[] shapes={"NOTEHEAD_VOID","NOTEHEAD_VOID_SMALL","NOTEHEAD_CROSS_VOID","NOTEHEAD_DIAMOND_VOID","NOTEHEAD_TRIANGLE_DOWN_VOID","NOTEHEAD_CIRCLE_X_VOID","NOTEHEAD_BLACK","NOTEHEAD_BLACK_SMALL","NOTEHEAD_CROSS","NOTEHEAD_DIAMOND_FILLED","NOTEHEAD_TRIANGLE_DOWN_FILLED","NOTEHEAD_CIRCLE_X"};
  try {
   call(scale,"setHeadSeedScale",(Object)null);Object catalog=call(factory,"getCatalog",en("ui.symbol.MusicFamily","Bravura"),51);
   for(String shape:shapes){Object sh=en("glyph.Shape",shape),template=call(catalog,"getTemplate",sh);Rectangle slim=(Rectangle)call(template,"getSlimBounds"),bounds=new Rectangle(40,50,slim.width,slim.height);
    for(int height:new int[]{3,7,15,20,30,31,40,64})for(String side:new String[]{"LEFT","RIGHT"})for(String v:new String[]{"TOP","BOTTOM"})for(int gap:new int[]{-2,0,1,4}){
     Object head=make("sig.inter.HeadInter",bounds,sh,.8,lastStaff,0.);setField(head,"template",template);Point2D ref=(Point2D)call(head,"getStemReferencePoint",en("util.HorizontalSide",side),en("util.VerticalSide",v));int x=(int)Math.rint(ref.getX())+(side.equals("LEFT")?-gap:gap)-1,y=v.equals("TOP")?(int)Math.rint(ref.getY())-height:(int)Math.rint(ref.getY());
     Object part=rectangleGlyph(x,y,2,height,"VERTICAL"),glyph=staticCall("glyph.GlyphFactory","buildGlyph",List.of(part)),stem=make("sig.inter.StemInter",glyph,.8);Line2D median=(Line2D)call(stem,"getMedian");Object r=staticCall("sig.relation.HeadStemRelation","checkRelation",head,median,glyph,en("util.VerticalSide",v),scale,0);String before=rel(r);int pv=((Collection<?>)call(sig,"vertexSet")).size(),pe=((Collection<?>)call(sig,"edgeSet")).size();Object ns=get(lastStaff,"notes");List notes=ns==null?new ArrayList():new ArrayList((Collection)ns);call(sig,"addVertex",head);call(sig,"addVertex",stem);if(r!=null){call(r,"setConsistency",head,stem);call(sig,"addEdge",head,stem,r);}String after=rel(r),state=state(sig,List.of(head),List.of(stem),pv,pe,notes);
     batches.add("{\"shape\":\""+shape+"\",\"small_head\":"+call(sh,"isSmallHead")+",\"head_bounds\":"+rect(bounds)+",\"staff_id\":"+call(lastStaff,"getId")+",\"main_interline\":"+call(scale,"getInterline")+",\"parts\":["+canonicalGlyph(part)+"],\"vertical\":\""+v+"\",\"glyph\":"+canonicalGlyph(glyph)+",\"median\":"+line(median)+",\"width\":"+call(stem,"getWidth")+",\"stem_bounds\":"+rect(call(stem,"getBounds"))+",\"before\":"+before+",\"relation\":"+after+",\"after\":"+state+"}");call(stem,"remove");call(head,"remove");
    }
   }
  }finally{call(scale,"setHeadSeedScale",savedScale);}
  Files.writeString(out.resolveSibling("stem-support-controls.json"),"{\"native_outputs_used\":false,\"scope\":\"Original HeadStemRelation.setConsistency(head,physicalStem), main-scale length and small-head support ratios; explicit selected parts, no automatic STEMS or contextual grade/cleaner/epilog claim.\",\"batches\":["+String.join(",",batches)+"]}\n");
 }
}
