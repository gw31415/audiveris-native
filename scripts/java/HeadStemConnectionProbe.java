// SPDX-License-Identifier: AGPL-3.0-or-later
// Original physical stem glyphs, reference/gap/connection and nonmanual SIG lifecycle.
import org.audiveris.omr.sheet.Book;import java.util.*;import java.nio.file.*;import java.awt.Rectangle;import java.awt.geom.*;
public class HeadStemConnectionProbe extends HeadRangeCatalogProbe {
 public HeadStemConnectionProbe(Book b,SortedSet<Integer> ids){super(b,ids);}
 String pt(Point2D p){return "["+p.getX()+","+p.getY()+"]";}
 String line(Line2D p){return "["+p.getX1()+","+p.getY1()+","+p.getX2()+","+p.getY2()+"]";}
 Object en(String cl,String s)throws Exception{return Class.forName("org.audiveris.omr."+cl).getField(s).get(null);}
 Object rectangleGlyph(int x,int y,int width,int height,String orientation)throws Exception{
  byte[] pixels=new byte[width*height];Object bp=Class.forName("ij.process.ByteProcessor").getConstructor(int.class,int.class,byte[].class).newInstance(width,height,pixels);
  Object table=call(make("run.RunTableFactory",en("run.Orientation",orientation)),"createTable",bp);
  return make("glyph.Glyph",x,y,table);
 }
 String rel(Object r)throws Exception {
  if(r==null)return "null";
  return "{\"head_side\":\""+call(r,"getHeadSide")+"\",\"extension_point\":"+pt((Point2D)call(r,"getExtensionPoint"))+",\"dx\":"+call(r,"getDx")+",\"dy\":"+call(r,"getDy")+",\"grade\":"+call(r,"getGrade")+",\"consistency\":"+call(r,"getConsistency")+",\"source_ratio\":"+call(r,"getSourceRatio")+",\"target_ratio\":"+call(r,"getTargetRatio")+",\"invading\":"+call(r,"isInvading")+"}";
 }
 String state(Object sig,List<Object> heads,List<Object> stems,int priorVertices,int priorEdges,List priorNotes)throws Exception {
  List<String> hs=new ArrayList<>(),ss=new ArrayList<>(),edges=new ArrayList<>();
  for(Object h:heads)hs.add("{\"removed\":"+call(h,"isRemoved")+",\"abnormal\":"+call(h,"isAbnormal")+"}");
  for(Object s:stems)ss.add("{\"removed\":"+call(s,"isRemoved")+",\"abnormal\":"+call(s,"isAbnormal")+"}");
  for(Object r:(Collection<?>)call(sig,"edgeSet"))if(r.getClass().getSimpleName().equals("HeadStemRelation")){Object a=call(sig,"getEdgeSource",r),b=call(sig,"getEdgeTarget",r);if(heads.contains(a)&&stems.contains(b))edges.add("["+(heads.indexOf(a)+1)+","+(stems.indexOf(b)+1)+"]");}
  int liveH=0,liveS=0;for(Object h:heads)if(!(Boolean)call(h,"isRemoved"))liveH++;for(Object s:stems)if(!(Boolean)call(s,"isRemoved"))liveS++;
  boolean prior=((Collection<?>)call(sig,"vertexSet")).size()==priorVertices+liveH+liveS&&((Collection<?>)call(sig,"edgeSet")).size()==priorEdges+edges.size();
  List<Object> notes=new ArrayList<>();Object ns=get(lastStaff,"notes");if(ns!=null)notes.addAll((Collection<?>)ns);List<Object> expected=new ArrayList<>(priorNotes);for(Object h:heads)if(!(Boolean)call(h,"isRemoved"))expected.add(h);
  return "{\"heads\":["+String.join(",",hs)+"],\"stems\":["+String.join(",",ss)+"],\"edges\":["+String.join(",",edges)+"],\"prior_SIG_preserved\":"+prior+",\"staff_notes_preserved\":"+notes.equals(expected)+"}";
 }
 void controls(Path out)throws Exception {
  Object sig=call(call(lastStaff,"getSystem"),"getSig"),scale=call(sheet,"getScale"),factory=staticCall("image.TemplateFactory","getInstance"),savedScale=call(scale,"getHeadSeedScale");
  List<String> references=new ArrayList<>(),gaps=new ArrayList<>(),connections=new ArrayList<>(),glyphs=new ArrayList<>(),lifecycle=new ArrayList<>(),batches=new ArrayList<>();
  String[] stems={"NOTEHEAD_VOID","NOTEHEAD_VOID_SMALL","NOTEHEAD_CROSS_VOID","NOTEHEAD_DIAMOND_VOID","NOTEHEAD_TRIANGLE_DOWN_VOID","NOTEHEAD_CIRCLE_X_VOID","NOTEHEAD_BLACK","NOTEHEAD_BLACK_SMALL","NOTEHEAD_CROSS","NOTEHEAD_DIAMOND_FILLED","NOTEHEAD_TRIANGLE_DOWN_FILLED","NOTEHEAD_CIRCLE_X"};
  try {
   call(scale,"setHeadSeedScale",(Object)null);
   for(String family:new String[]{"Bravura","Leland","FinaleJazz"})for(int size:new int[]{32,40,51,64})for(String shape:stems){
    Object catalog=call(factory,"getCatalog",en("ui.symbol.MusicFamily",family),size),sh=en("glyph.Shape",shape),template=call(catalog,"getTemplate",sh);Rectangle slim=(Rectangle)call(template,"getSlimBounds");
    for(int variant=0;variant<3;variant++){
     Rectangle bounds=new Rectangle(40-variant*3,50-variant*2,Math.max(1,slim.width-variant),Math.max(1,slim.height-variant));Object head=make("sig.inter.HeadInter",bounds,sh,.8,lastStaff,0.);setField(head,"template",template);
     for(String side:new String[]{"LEFT","RIGHT"})for(String vertical:new String[]{"TOP","BOTTOM"})references.add("{\"family\":\""+family+"\",\"point_size\":"+size+",\"shape\":\""+shape+"\",\"bounds\":"+rect(bounds)+",\"side\":\""+side+"\",\"vertical\":\""+vertical+"\",\"learned_dx\":[null,null],\"point\":"+pt((Point2D)call(head,"getStemReferencePoint",en("util.HorizontalSide",side),en("util.VerticalSide",vertical)))+"}");
     if(variant==0&&(shape.equals("NOTEHEAD_BLACK")||shape.equals("NOTEHEAD_VOID"))){call(sig,"addVertex",head);for(int mode=0;mode<4;mode++){Object learned=make("sheet.note.HeadSeedScale");Double l=mode==0?null:mode==1?.5:mode==3?-.75:.5,r=mode==0?null:mode==2?null:mode==3?1.25:.25;if(l!=null)call(learned,"putDx",sh,en("util.HorizontalSide","LEFT"),l);if(r!=null)call(learned,"putDx",sh,en("util.HorizontalSide","RIGHT"),r);call(scale,"setHeadSeedScale",learned);for(String side:new String[]{"LEFT","RIGHT"})for(String vertical:new String[]{"TOP","BOTTOM"})references.add("{\"family\":\""+family+"\",\"point_size\":"+size+",\"shape\":\""+shape+"\",\"bounds\":"+rect(bounds)+",\"side\":\""+side+"\",\"vertical\":\""+vertical+"\",\"learned_dx\":["+l+","+r+"],\"point\":"+pt((Point2D)call(head,"getStemReferencePoint",en("util.HorizontalSide",side),en("util.VerticalSide",vertical)))+"}");}call(scale,"setHeadSeedScale",(Object)null);call(head,"remove");}
    }
   }
   for(int profile:new int[]{0,1,2})for(double dx:new double[]{-.5,-.4,-.2,-1e-9,0.,.05-1e-9,.05,.05+1e-9,.15,.25,.35,.4})for(double dy:new double[]{-.1,0.,1e-9,.1,.4,.8,1.2,1.3}){Object r=make("sig.relation.HeadStemRelation");call(r,"setInOutGaps",dx,dy,profile);Object impacts=call(r,"getImpacts");gaps.add("{\"profile\":"+profile+",\"dx\":"+dx+",\"dy\":"+dy+",\"grade\":"+call(r,"getGrade")+",\"x_impact\":"+call(impacts,"getImpact",0)+",\"y_impact\":"+call(impacts,"getImpact",1)+",\"source_ratio\":"+call(r,"getSourceRatio")+",\"target_ratio\":"+call(r,"getTargetRatio")+",\"eligible\":"+((Double)call(r,"getGrade")>=(Double)call(r,"getMinGrade"))+"}");}
   Object catalog=call(factory,"getCatalog",en("ui.symbol.MusicFamily","Bravura"),51);
   for(String shape:stems){Object sh=en("glyph.Shape",shape),template=call(catalog,"getTemplate",sh);Rectangle slim=(Rectangle)call(template,"getSlimBounds"),bounds=new Rectangle(40,50,slim.width,slim.height);Object head=make("sig.inter.HeadInter",bounds,sh,.8,lastStaff,0.);setField(head,"template",template);
    for(String side:new String[]{"LEFT","RIGHT"})for(String v:new String[]{"TOP","BOTTOM"}){Point2D ref=(Point2D)call(head,"getStemReferencePoint",en("util.HorizontalSide",side),en("util.VerticalSide",v));for(int dx:new int[]{-4,-2,0,2,4})for(int dy:new int[]{-3,0,3,10})for(boolean physical:new boolean[]{false,true})for(int profile:new int[]{0,1,2}){
     double x=ref.getX()+(side.equals("LEFT")?-dx:dx),tilt=(dx%3)*.05;double y1=v.equals("TOP")?ref.getY()-40:ref.getY()+dy,y2=v.equals("TOP")?ref.getY()-dy:ref.getY()+40;Line2D median=new Line2D.Double(x+tilt,y1,x-tilt,y2);Object glyph=physical?rectangleGlyph((int)Math.floor(x), (int)Math.floor(y1),2,Math.max(1,(int)Math.ceil(y2-y1)),"VERTICAL"):null;
     Object r=staticCall("sig.relation.HeadStemRelation","checkRelation",head,median,glyph,en("util.VerticalSide",v),scale,profile);
     connections.add("{\"shape\":\""+shape+"\",\"bounds\":"+rect(bounds)+",\"stem\":"+line(median)+",\"glyph_bounds\":"+(glyph==null?"null":rect(call(glyph,"getBounds")))+",\"vertical\":\""+v+"\",\"profile\":"+profile+",\"relation\":"+rel(r)+"}");
    }}
   }
   for(String orientation:new String[]{"VERTICAL","HORIZONTAL"})for(int height:new int[]{2,7,20,35,64})for(int mode=0;mode<4;mode++){
    Object a=rectangleGlyph(50,20,2,height,orientation),b=rectangleGlyph(mode==2?51:50,mode==3?20:20+height+2,2,height,orientation);List<Object> parts=mode==0?List.of(a):mode==3?List.of(a,a):List.of(a,b);Object glyph=staticCall("glyph.GlyphFactory","buildGlyph",parts),stem=make("sig.inter.StemInter",glyph,.8);List<String> inputs=new ArrayList<>();for(Object p:parts)inputs.add(canonicalGlyph(p));
    glyphs.add("{\"parts\":["+String.join(",",inputs)+"],\"glyph\":"+canonicalGlyph(glyph)+",\"median\":"+line((Line2D)call(stem,"getMedian"))+",\"width\":"+call(stem,"getWidth")+",\"bounds\":"+rect(call(stem,"getBounds"))+",\"single_part_identity_preserved\":"+(mode!=0||glyph==a)+"}");
   }
   for(String shape:stems){Object sh=en("glyph.Shape",shape),template=call(catalog,"getTemplate",sh);Rectangle slim=(Rectangle)call(template,"getSlimBounds");Rectangle bounds=new Rectangle(40,50,slim.width,slim.height);
    for(String side:new String[]{"LEFT","RIGHT"})for(String v:new String[]{"TOP","BOTTOM"})for(int gap:new int[]{-3,0,3}){
     Object head=make("sig.inter.HeadInter",bounds,sh,.8,lastStaff,0.);setField(head,"template",template);Point2D ref=(Point2D)call(head,"getStemReferencePoint",en("util.HorizontalSide",side),en("util.VerticalSide",v));int x=(int)Math.rint(ref.getX())+(side.equals("LEFT")?-gap:gap)-1,y=v.equals("TOP")?20:50;Object part=rectangleGlyph(x,y,2,40,"VERTICAL"),glyph=staticCall("glyph.GlyphFactory","buildGlyph",List.of(part)),stem=make("sig.inter.StemInter",glyph,.8);Line2D median=(Line2D)call(stem,"getMedian");Object r=staticCall("sig.relation.HeadStemRelation","checkRelation",head,median,glyph,en("util.VerticalSide",v),scale,0);String relation=rel(r);int pv=((Collection<?>)call(sig,"vertexSet")).size(),pe=((Collection<?>)call(sig,"edgeSet")).size();Object ns=get(lastStaff,"notes");List notes=ns==null?new ArrayList():new ArrayList((Collection)ns);call(sig,"addVertex",head);call(sig,"addVertex",stem);if(r!=null)call(sig,"addEdge",head,stem,r);String after=state(sig,List.of(head),List.of(stem),pv,pe,notes);
     batches.add("{\"shape\":\""+shape+"\",\"head_bounds\":"+rect(bounds)+",\"staff_id\":"+call(lastStaff,"getId")+",\"parts\":["+canonicalGlyph(part)+"],\"vertical\":\""+v+"\",\"glyph\":"+canonicalGlyph(glyph)+",\"median\":"+line(median)+",\"width\":"+call(stem,"getWidth")+",\"stem_bounds\":"+rect(call(stem,"getBounds"))+",\"relation\":"+relation+",\"after\":"+after+"}");call(stem,"remove");call(head,"remove");
    }
   }
   for(String shape:stems){Object sh=en("glyph.Shape",shape),template=call(catalog,"getTemplate",sh);Rectangle sb=(Rectangle)call(template,"getSlimBounds");Object h1=make("sig.inter.HeadInter",new Rectangle(40,50,sb.width,sb.height),sh,.8,lastStaff,0.),h2=make("sig.inter.HeadInter",new Rectangle(40,70,sb.width,sb.height),sh,.8,lastStaff,0.);setField(h1,"template",template);setField(h2,"template",template);Object glyph=rectangleGlyph(53,20,2,65,"VERTICAL"),stem=make("sig.inter.StemInter",glyph,.8);List<Object> hs=List.of(h1,h2),ss=List.of(stem);int pv=((Collection<?>)call(sig,"vertexSet")).size(),pe=((Collection<?>)call(sig,"edgeSet")).size();Object ns=get(lastStaff,"notes");List notes=ns==null?new ArrayList():new ArrayList((Collection)ns);List<String> states=new ArrayList<>();
    call(sig,"addVertex",h1);call(sig,"addVertex",h2);call(sig,"addVertex",stem);states.add(state(sig,hs,ss,pv,pe,notes));
    Object r1=make("sig.relation.HeadStemRelation"),r2=make("sig.relation.HeadStemRelation"),r3=make("sig.relation.HeadStemRelation");for(Object r:List.of(r1,r2,r3)){call(r,"setHeadSide",en("util.HorizontalSide","RIGHT"));call(r,"setExtensionPoint",new Point2D.Double(54,50));call(r,"setInOutGaps",0.,0.,0);}
    call(sig,"addEdge",h1,stem,r1);states.add(state(sig,hs,ss,pv,pe,notes));call(sig,"addEdge",h2,stem,r2);states.add(state(sig,hs,ss,pv,pe,notes));boolean parallel=(Boolean)call(sig,"addEdge",h1,stem,r3);states.add(state(sig,hs,ss,pv,pe,notes));call(sig,"removeEdge",r1);states.add(state(sig,hs,ss,pv,pe,notes));if(parallel)call(sig,"removeEdge",r3);states.add(state(sig,hs,ss,pv,pe,notes));call(stem,"remove");states.add(state(sig,hs,ss,pv,pe,notes));call(h1,"remove");call(h2,"remove");states.add(state(sig,hs,ss,pv,pe,notes));lifecycle.add("{\"head_bounds\":["+rect(call(h1,"getBounds"))+","+rect(call(h2,"getBounds"))+"],\"staff_id\":"+call(lastStaff,"getId")+",\"shape\":\""+shape+"\",\"parallel_relation_accepted\":"+parallel+",\"states\":["+String.join(",",states)+"]}");
   }
  }finally{call(scale,"setHeadSeedScale",savedScale);}
  Files.writeString(out.resolveSibling("stem-connection-controls.json"),"{\"native_outputs_used\":false,\"scope\":\"Original explicit stem glyph parts and nonmanual relation geometry/SIG lifecycle; no automatic STEMS assembly, chords/manual links, cleaner/epilog or full STEMS claim.\",\"minimum_grade\":"+call(make("sig.relation.HeadStemRelation"),"getMinGrade")+",\"references\":["+String.join(",",references)+"],\"gaps\":["+String.join(",",gaps)+"],\"connections\":["+String.join(",",connections)+"],\"glyphs\":["+String.join(",",glyphs)+"],\"lifecycle\":["+String.join(",",lifecycle)+"],\"batches\":["+String.join(",",batches)+"]}\n");
 }
}
