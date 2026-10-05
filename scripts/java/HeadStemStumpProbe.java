// SPDX-License-Identifier: AGPL-3.0-or-later
// Original HeadLinker corner-stump argument on explicitly selected stem unions.
import org.audiveris.omr.sheet.Book;import java.util.*;import java.nio.file.*;import java.awt.Rectangle;import java.awt.geom.*;
public class HeadStemStumpProbe extends HeadStemSupportProbe {
 public HeadStemStumpProbe(Book b,SortedSet<Integer> ids){super(b,ids);}
 @Override void controls(Path out)throws Exception {
  Object sig=call(call(lastStaff,"getSystem"),"getSig"),scale=call(sheet,"getScale"),factory=staticCall("image.TemplateFactory","getInstance"),savedScale=call(scale,"getHeadSeedScale");List<String> batches=new ArrayList<>();
  String[] shapes={"NOTEHEAD_VOID","NOTEHEAD_VOID_SMALL","NOTEHEAD_CROSS_VOID","NOTEHEAD_DIAMOND_VOID","NOTEHEAD_TRIANGLE_DOWN_VOID","NOTEHEAD_CIRCLE_X_VOID","NOTEHEAD_BLACK","NOTEHEAD_BLACK_SMALL","NOTEHEAD_CROSS","NOTEHEAD_DIAMOND_FILLED","NOTEHEAD_TRIANGLE_DOWN_FILLED","NOTEHEAD_CIRCLE_X"};
  try {
   call(scale,"setHeadSeedScale",(Object)null);Object catalog=call(factory,"getCatalog",en("ui.symbol.MusicFamily","Bravura"),51);
   for(String shape:shapes){Object sh=en("glyph.Shape",shape),template=call(catalog,"getTemplate",sh);Rectangle slim=(Rectangle)call(template,"getSlimBounds"),bounds=new Rectangle(40,50,slim.width,slim.height);
    for(String side:new String[]{"LEFT","RIGHT"})for(String v:new String[]{"TOP","BOTTOM"})for(int gap:new int[]{-1,0,1,3,9,10,15})for(int dx:new int[]{-2,0,2})for(boolean physical:new boolean[]{false,true})for(int profile:new int[]{0,1,2}){
     Object head=make("sig.inter.HeadInter",bounds,sh,.8,lastStaff,0.);setField(head,"template",template);Point2D ref=(Point2D)call(head,"getStemReferencePoint",en("util.HorizontalSide",side),en("util.VerticalSide",v));int x=(int)Math.rint(ref.getX())+(side.equals("LEFT")?-dx:dx)-1,y=v.equals("TOP")?(int)Math.rint(ref.getY())-gap-30:(int)Math.rint(ref.getY())+gap;
     Object a=rectangleGlyph(x,y,2,12,"VERTICAL"),b=rectangleGlyph(x,y+18,2,12,"VERTICAL"),glyph=staticCall("glyph.GlyphFactory","buildGlyph",List.of(a,b)),stem=make("sig.inter.StemInter",glyph,.8),stump=physical?rectangleGlyph(x,(int)Math.rint(ref.getY())-(v.equals("TOP")?1:2),2,3,"VERTICAL"):null;Line2D median=(Line2D)call(stem,"getMedian");Object r=staticCall("sig.relation.HeadStemRelation","checkRelation",head,median,stump,en("util.VerticalSide",v),scale,profile);String before=rel(r),union=rel(staticCall("sig.relation.HeadStemRelation","checkRelation",head,median,glyph,en("util.VerticalSide",v),scale,profile));int pv=((Collection<?>)call(sig,"vertexSet")).size(),pe=((Collection<?>)call(sig,"edgeSet")).size();Object ns=get(lastStaff,"notes");List notes=ns==null?new ArrayList():new ArrayList((Collection)ns);call(sig,"addVertex",head);call(sig,"addVertex",stem);if(r!=null){call(r,"setConsistency",head,stem);call(sig,"addEdge",head,stem,r);}String after=rel(r),state=state(sig,List.of(head),List.of(stem),pv,pe,notes);
     batches.add("{\"shape\":\""+shape+"\",\"head_bounds\":"+rect(bounds)+",\"staff_id\":"+call(lastStaff,"getId")+",\"main_interline\":"+call(scale,"getInterline")+",\"parts\":["+canonicalGlyph(a)+","+canonicalGlyph(b)+"],\"stump\":"+(stump==null?"null":canonicalGlyph(stump))+",\"vertical\":\""+v+"\",\"profile\":"+profile+",\"glyph\":"+canonicalGlyph(glyph)+",\"median\":"+line(median)+",\"width\":"+call(stem,"getWidth")+",\"stem_bounds\":"+rect(call(stem,"getBounds"))+",\"union_contact_relation\":"+union+",\"before\":"+before+",\"relation\":"+after+",\"after\":"+state+"}");call(stem,"remove");call(head,"remove");
    }
   }
  }finally{call(scale,"setHeadSeedScale",savedScale);}
  Files.writeString(out.resolveSibling("stem-stump-controls.json"),"{\"native_outputs_used\":false,\"scope\":\"Original checkRelation head-corner stump argument vs selected two-part stem union, followed by physical consistency and nonmanual SIG registration. Stump selection and automatic StemBuilder remain unported.\",\"batches\":["+String.join(",",batches)+"]}\n");
 }
}
