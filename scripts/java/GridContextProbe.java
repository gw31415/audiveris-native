// SPDX-License-Identifier: AGPL-3.0-or-later
// Controlled SIGraph cases on original official objects; not image-derived exclusions.
import org.audiveris.omr.sheet.Book;import java.util.*;import java.nio.file.*;
public class GridContextProbe extends GridProbe {
 private final Object book;public GridContextProbe(Book b,SortedSet<Integer> ids){super(b,ids);book=b;}
 static String run(Object system,String name,double[]grades,double[][]supports,int[][]exclusions)throws Exception{
  Object sig=Class.forName("org.audiveris.omr.sig.SIGraph").getConstructor(Class.forName("org.audiveris.omr.sheet.SystemInfo")).newInstance(system);
  Class<?> bar=Class.forName("org.audiveris.omr.sig.inter.BarlineInter"),shape=Class.forName("org.audiveris.omr.glyph.Shape"),glyph=Class.forName("org.audiveris.omr.glyph.Glyph");
  var constructor=bar.getConstructor(glyph,shape,Double.class,java.awt.geom.Line2D.class,Double.class);
  List<Object>nodes=new ArrayList<>();List<String>inters=new ArrayList<>(),rels=new ArrayList<>();
  for(int i=0;i<grades.length;i++){Object node=constructor.newInstance(null,shape.getField("THIN_BARLINE").get(null),grades[i],new java.awt.geom.Line2D.Double(10+i*10,10,10+i*10,50),2.0);call(sig,"addVertex",node);nodes.add(node);}
  for(double[]s:supports){Object rel=Class.forName("org.audiveris.omr.sig.relation.BarConnectionRelation").getConstructor().newInstance();call(rel,"setGrade",s[2]);call(sig,"addEdge",nodes.get((int)s[0]),nodes.get((int)s[1]),rel);rels.add("{\"kind\":\"BarConnectionRelation\",\"source\":"+(int)s[0]+",\"target\":"+(int)s[1]+",\"grade\":"+s[2]+"}");}
  Class<?>cause=Class.forName("org.audiveris.omr.sig.relation.Exclusion$ExclusionCause");
  for(int[]e:exclusions){Object rel=Class.forName("org.audiveris.omr.sig.relation.Exclusion").getConstructor(cause).newInstance(cause.getField("OVERLAP").get(null));call(sig,"addEdge",nodes.get(e[0]),nodes.get(e[1]),rel);rels.add("{\"kind\":\"Exclusion\",\"source\":"+e[0]+",\"target\":"+e[1]+"}");}
  for(int i=0;i<nodes.size();i++)inters.add("{\"id\":"+i+",\"kind\":\"BarlineInter\",\"grade\":"+grades[i]+",\"contextual_grade\":"+call(sig,"computeContextualGrade",nodes.get(i))+"}");
  return "{\"name\":\""+name+"\",\"inters\":["+String.join(",",inters)+"],\"relations\":["+String.join(",",rels)+"]}";
 }
 public void process(){try{
 Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");call(Class.forName("org.audiveris.omr.sheet.grid.GridStep").getConstructor().newInstance(),"doit",sheet);Object system=((List<?>)call(sheet,"getSystems")).get(0);List<String>cases=new ArrayList<>();
 cases.add(run(system,"isolated",new double[]{.4},new double[][]{},new int[][]{}));
 cases.add(run(system,"ordered-supports",new double[]{.31,.2,.7,.4},new double[][]{{0,1,.7},{2,0,.9},{0,3,.1}},new int[][]{}));
 cases.add(run(system,"exclusive-pair",new double[]{.3,.6,.5},new double[][]{{0,1,.8},{2,0,.9}},new int[][]{{1,2}}));
 cases.add(run(system,"exclusion-chain",new double[]{.4,.7,.5,.6,.3},new double[][]{{0,1,.5},{0,2,.9},{3,0,.8},{0,4,.95}},new int[][]{{1,2},{2,3},{3,4}}));
 cases.add(run(system,"complete-conflict",new double[]{.4,.6,.6,.6},new double[][]{{0,1,.4},{0,2,.8},{0,3,.9}},new int[][]{{1,2},{1,3},{2,3}}));
 cases.add(run(system,"duplicate-partners",new double[]{.4,.6,.5},new double[][]{{0,1,.2},{1,0,.9},{0,2,.7}},new int[][]{{1,2}}));
 cases.add(run(system,"zero-support",new double[]{.4,.7,.5},new double[][]{{0,1,0},{2,0,.7}},new int[][]{{1,2}}));
 Files.writeString(Path.of(System.getProperty("native.probe.output")),"["+String.join(",",cases)+"]\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
