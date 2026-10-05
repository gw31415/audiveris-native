// SPDX-License-Identifier: AGPL-3.0-or-later
// Development oracle: original GlyphCluster on the two actual native residue
// rasters. A kernel comparison, not an original full-page SymbolsFilter trace.
import java.lang.reflect.*;import java.util.*;import java.nio.file.*;
public class LedgerSymbolClusterProbe {
 static Class<?> c(String n)throws Exception{return Class.forName("org.audiveris.omr."+n);}
 static Object call(Object o,String n,Object...a)throws Exception{for(Method m:o.getClass().getMethods())if(m.getName().equals(n)&&m.getParameterCount()==a.length){try{return m.invoke(o,a);}catch(IllegalArgumentException e){}}throw new NoSuchMethodException(n);}
 @SuppressWarnings({"unchecked","rawtypes"}) static Object en(String c,String n)throws Exception{return Enum.valueOf((Class)LedgerSymbolClusterProbe.c(c),n);}
 static Object glyph(int x,int y,int[][] runs)throws Exception{byte[] p=new byte[6];Arrays.fill(p,(byte)255);for(int[] r:runs)for(int yy=r[1];yy<r[1]+r[2];yy++)p[yy*2+r[0]]=0;Object bp=Class.forName("ij.process.ByteProcessor").getConstructor(int.class,int.class,byte[].class).newInstance(2,3,p),f=c("run.RunTableFactory").getConstructor(c("run.Orientation")).newInstance(en("run.Orientation","VERTICAL"));return c("glyph.Glyph").getConstructor(int.class,int.class,c("run.RunTable")).newInstance(x,y,call(f,"createTable",bp));}
 public static void main(String[] a)throws Exception{
 Object one=glyph(272,277,new int[][]{{0,0,2},{1,0,3}}),two=glyph(273,272,new int[][]{{0,1,2},{1,0,3}});List<Object> parts=new ArrayList<>(List.of(one,two));Object links=c("glyph.Glyphs").getMethod("buildLinks",Collection.class,double.class).invoke(null,parts,8.8);Object classifier=c("classifier.BasicClassifier").getMethod("getInstance").invoke(null);List<String> trials=new ArrayList<>();
 Object adapter=Proxy.newProxyInstance(LedgerSymbolClusterProbe.class.getClassLoader(),new Class[]{c("glyph.GlyphCluster$GlyphAdapter")},(p,m,v)->switch(m.getName()){
 case "getParts"->new ArrayList<>(parts);
 case "getNeighbors"->new ArrayList<>(List.of(v[0]==one?two:one));
 case "isTooHeavy","isTooLarge"->false;
 case "isTooLight"->((Integer)v[0])/121.0<0.04;
 case "evaluateGlyph"->{Object g=v[0];Object[] ev=(Object[])call(classifier,"getNaturalEvaluations",g,11);List<String> accepted=new ArrayList<>();for(Object e:ev){double grade=e.getClass().getField("grade").getDouble(e);if(grade>=0.15)accepted.add("{\"shape\":\""+e.getClass().getField("shape").get(e)+"\",\"score\":"+grade+"}");}java.awt.Rectangle b=(java.awt.Rectangle)call(g,"getBounds");trials.add("{\"parts\":"+((Set<?>)v[1]).size()+",\"bounds\":["+b.x+","+b.y+","+b.width+","+b.height+"],\"weight\":"+call(g,"getWeight")+",\"evaluations_above_0_15\":["+String.join(",",accepted)+"]}");yield null;}
 default->throw new UnsupportedOperationException(m.getName());});
 Object cluster=c("glyph.GlyphCluster").getConstructor(c("glyph.GlyphCluster$GlyphAdapter"),c("glyph.GlyphGroup")).newInstance(adapter,en("glyph.GlyphGroup","SYMBOL"));call(cluster,"decompose");
 Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(a[1]));Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),staff=((List<?>)call(sys,"getStaves")).get(0);Object union=c("glyph.GlyphFactory").getMethod("buildGlyph",Collection.class).invoke(null,parts);Object dots=c("sheet.symbol.DotFactory").getConstructor(c("sheet.symbol.InterFactory"),c("sheet.SystemInfo")).newInstance(null,sys);Method concrete=c("sheet.symbol.DotFactory").getDeclaredMethod("checkDistanceToConcreteLine",c("glyph.Glyph"),c("sheet.Staff"));concrete.setAccessible(true);boolean allowed=(Boolean)concrete.invoke(dots,union,staff);if(allowed)throw new IllegalStateException("Expected actual ledger to reject residue dot");
 Files.writeString(Path.of(a[0]),"{\"scope\":\"original GlyphCluster/BasicClassifier kernel on native actual C4 residual runs; not independent original raster trace\",\"concrete_ledger_dot_allowed\":"+allowed+",\"original_link_count\":"+((Set<?>)call(links,"edgeSet")).size()+",\"max_gap\":8.8,\"interline\":11,\"trials\":["+String.join(",",trials)+"]}\n");
 }
}
