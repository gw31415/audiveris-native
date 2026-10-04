// SPDX-License-Identifier: AGPL-3.0-or-later
// Original selected physical stems and actual SIG cleaner. No automatic parts claim.
import org.audiveris.omr.sheet.Book;import java.util.*;import java.nio.file.*;import java.awt.Rectangle;import java.awt.geom.*;
public class HeadStemsCleanerControlProbe extends HeadStemConnectionProbe {
 public HeadStemsCleanerControlProbe(Book b,SortedSet<Integer> ids){super(b,ids);}
 @Override void controls(Path out)throws Exception {
  Object sig=call(call(lastStaff,"getSystem"),"getSig"),factory=staticCall("image.TemplateFactory","getInstance"),catalog=call(factory,"getCatalog",en("ui.symbol.MusicFamily","Bravura"),51);List<String> cases=new ArrayList<>();
  for(String shape:new String[]{"NOTEHEAD_BLACK","NOTEHEAD_BLACK_SMALL"})for(int count=0;count<=4;count++)for(String mode:new String[]{"canonical","same-side","wrong-direction","midpoint","missing"})for(boolean reverse:new boolean[]{false,true})for(boolean tie:new boolean[]{false,true})for(double dy:new double[]{0.,.2-1e-9,.2,.2+1e-9}){
   Object sh=en("glyph.Shape",shape),template=call(catalog,"getTemplate",sh);Rectangle slim=(Rectangle)call(template,"getSlimBounds"),bounds=new Rectangle(40,50,slim.width,slim.height);Object head=make("sig.inter.HeadInter",bounds,sh,.8,lastStaff,0.);setField(head,"template",template);call(sig,"addVertex",head);List<Object> stems=new ArrayList<>(),relations=new ArrayList<>();List<String> inputs=new ArrayList<>();int center=bounds.y+bounds.height/2;
   for(int j=0;j<count;j++){
    String side=j%2==0?"LEFT":"RIGHT";if(mode.equals("same-side"))side="LEFT";
    int y=side.equals("LEFT")?bounds.y:bounds.y+bounds.height-1-40;if(mode.equals("wrong-direction"))y=side.equals("LEFT")?bounds.y-40:bounds.y+bounds.height-1;if(mode.equals("midpoint"))y=center-20;
    Object glyph=rectangleGlyph(40+j*5,y,2,40,"VERTICAL"),stem=make("sig.inter.StemInter",glyph,tie?.8:j%2==0?.2:.8);call(sig,"addVertex",stem);stems.add(stem);Object r=null;
    if(!mode.equals("missing")||j!=count-1){r=make("sig.relation.HeadStemRelation");call(r,"setHeadSide",en("util.HorizontalSide",side));call(r,"setExtensionPoint",new Point2D.Double(41+j*5,bounds.y));call(r,"setInOutGaps",0.,dy,0);call(sig,"addEdge",head,stem,r);}relations.add(r);
    inputs.add("{\"parts\":["+canonicalGlyph(glyph)+"],\"grade\":"+call(stem,"getGrade")+",\"median\":"+line((Line2D)call(stem,"getMedian"))+",\"width\":"+call(stem,"getWidth")+",\"bounds\":"+rect(call(stem,"getBounds"))+",\"relation\":"+rel(r)+"}");
   }
   List<Object> ordered=new ArrayList<>(stems);if(reverse)Collections.reverse(ordered);List<String> order=new ArrayList<>();for(Object s:ordered)order.add(""+stems.indexOf(s));int beforeVertices=((Collection<?>)call(sig,"vertexSet")).size(),beforeEdges=((Collection<?>)call(sig,"edgeSet")).size();call(make("sheet.stem.HeadStemsCleaner",head),"check",ordered);List<String> kept=new ArrayList<>(),abnormal=new ArrayList<>(),removed=new ArrayList<>();for(int j=0;j<count;j++){Object r=relations.get(j);if(r!=null&&(Boolean)call(sig,"containsEdge",r))kept.add(""+j);abnormal.add(""+call(stems.get(j),"isAbnormal"));removed.add(""+call(stems.get(j),"isRemoved"));}
   cases.add("{\"shape\":\""+shape+"\",\"head_bounds\":"+rect(bounds)+",\"mode\":\""+mode+"\",\"dy\":"+dy+",\"order\":["+String.join(",",order)+"],\"stems\":["+String.join(",",inputs)+"],\"kept\":["+String.join(",",kept)+"],\"head_abnormal\":"+call(head,"isAbnormal")+",\"stem_abnormal\":["+String.join(",",abnormal)+"],\"stem_removed\":["+String.join(",",removed)+"],\"vertex_delta\":"+(((Collection<?>)call(sig,"vertexSet")).size()-beforeVertices)+",\"edge_delta\":"+(((Collection<?>)call(sig,"edgeSet")).size()-beforeEdges)+"}");for(Object stem:stems)call(stem,"remove");call(head,"remove");
  }
  Files.writeString(out.resolveSibling("stem-cleaner-controls.json"),"{\"native_outputs_used\":false,\"scope\":\"Original selected physical HeadStemsCleaner.check, actual SIG edge removals and canonical sharing; not automatic parts or epilog parity\",\"tiny_y_gap\":0.2,\"cases\":["+String.join(",",cases)+"]}\n");
 }
}
