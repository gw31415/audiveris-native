// SPDX-License-Identifier: AGPL-3.0-or-later
// Original controlled focused-head getPartitions and parallel support cases.
import org.audiveris.omr.sheet.Book;
import java.util.*;
import java.nio.file.*;
import java.awt.Rectangle;
import java.awt.geom.Point2D;
public class HeadStemContextControlsProbe extends StemContextBoundaryProbe {
 private final Object book;
 public HeadStemContextControlsProbe(Book b,SortedSet<Integer> ids){super(b,ids);book=b;}
 public void process(){try{
  Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),staff=((List<?>)call(sys,"getStaves")).get(0);
  Class<?> sc=Class.forName("org.audiveris.omr.sig.SIGraph"),hc=Class.forName("org.audiveris.omr.sig.inter.HeadInter"),tc=Class.forName("org.audiveris.omr.sig.inter.StemInter"),gc=Class.forName("org.audiveris.omr.glyph.Glyph");List<String> cases=new ArrayList<>();
  for(int count=0;count<=4;count++)for(boolean reverse:new boolean[]{false,true})for(boolean parallel:new boolean[]{false,true})for(boolean tie:new boolean[]{false,true}){
   Object sig=sc.getConstructor(Class.forName("org.audiveris.omr.sheet.SystemInfo")).newInstance(sys);
   Object head=hc.getConstructor(Rectangle.class,Class.forName("org.audiveris.omr.glyph.Shape"),Double.class,Class.forName("org.audiveris.omr.sheet.Staff"),Double.class).newInstance(new Rectangle(100,100,12,8),en("glyph.Shape","NOTEHEAD_BLACK"),.4,staff,0.);call(sig,"addVertex",head);
   List<Object> nodes=new ArrayList<>(),stems=new ArrayList<>();nodes.add(head);List<String> edges=new ArrayList<>();
   for(int i=0;i<count;i++){Object stem=tc.getConstructor(gc,Double.class).newInstance(glyph(110+i*5,65,2,40,"VERTICAL"),tie?.6:.2+i*.2);call(sig,"addVertex",stem);stems.add(stem);nodes.add(stem);}
   List<Object> order=new ArrayList<>(stems);if(reverse)Collections.reverse(order);
   for(Object stem:order)for(int copy=0;copy<(parallel?2:1);copy++){
    Object r=Class.forName("org.audiveris.omr.sig.relation.HeadStemRelation").getConstructor().newInstance();call(r,"setHeadSide",en("util.HorizontalSide","RIGHT"));call(r,"setExtensionPoint",new Point2D.Double(110,107));call(r,"setInOutGaps",0.,copy==0?0.:.15,0);call(sig,"addEdge",head,stem,r);
    edges.add("{\"kind\":\"HeadStemRelation\",\"source\":1,\"target\":"+(nodes.indexOf(stem)+1)+",\"source_ratio\":"+call(r,"getSourceRatio")+",\"target_ratio\":"+call(r,"getTargetRatio")+"}");
   }
   call(sig,"contextualize");List<String> rows=new ArrayList<>();for(int i=0;i<nodes.size();i++){Object v=nodes.get(i);rows.add("{\"id\":"+(i+1)+",\"kind\":\""+v.getClass().getSimpleName()+"\",\"grade\":"+call(v,"getGrade")+",\"contextual_grade\":"+call(v,"getContextualGrade")+"}");}
   cases.add("{\"stem_count\":"+count+",\"reverse\":"+reverse+",\"parallel\":"+parallel+",\"tie\":"+tie+",\"inters\":["+String.join(",",rows)+"],\"relations\":["+String.join(",",edges)+"]}");for(Object v:nodes)call(v,"remove");
  }
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"native_outputs_used\":false,\"scope\":\"Original controlled focused-head concurrent stem partners and parallel support map overwrite; no image selection claim\",\"cases\":["+String.join(",",cases)+"]}");
 }catch(Exception e){throw new RuntimeException(e);}}
}
