// SPDX-License-Identifier: AGPL-3.0-or-later
// Development-only calls to original BlackHeadSizer methods on independent,
// deterministic synthetic glyphs. No native outputs are inputs.
import org.audiveris.omr.sheet.Book;
import java.util.*;
import java.nio.file.*;
import java.lang.reflect.*;
public class BlackHeadSizerControlProbe extends GridFinalProbe {
 private final Object book;
 public BlackHeadSizerControlProbe(Book b,SortedSet<Integer> ids){super(b,ids);book=b;}
 static Class<?> cls(String n)throws Exception{return Class.forName(n);}
 static Object construct(String n,Class<?>[] types,Object...args)throws Exception{Constructor<?> c=cls(n).getDeclaredConstructor(types);c.setAccessible(true);return c.newInstance(args);}
 static Object en(String n,String v)throws Exception{return cls(n).getField(v).get(null);}
 static String q(Object v){return v==null?"null":"\""+v.toString().replace("\\","\\\\").replace("\"","\\\"")+"\"";}
 static String fields(Object p)throws Exception{if(p==null)return "null";List<String> out=new ArrayList<>();for(Field f:p.getClass().getDeclaredFields())if(!Modifier.isStatic(f.getModifiers())){f.setAccessible(true);Object v=f.get(p);if(v==null||v instanceof Number||v instanceof Boolean||v instanceof String)out.add(q(f.getName())+":"+(v instanceof String?q(v):v));}return "{"+String.join(",",out)+"}";}
 static String canonical(Object g)throws Exception{return g==null?"null":glyph(g).replaceFirst("\\\"id\\\":-?[0-9]+,","");}
 static String groups(Object g)throws Exception{List<String> out=new ArrayList<>();for(Object v:(Collection<?>)call(g,"getGroups"))out.add(q(v));Collections.sort(out);return "["+String.join(",",out)+"]";}
 static Object scale(int n)throws Exception {
  Object il=construct("org.audiveris.omr.sheet.Scale$InterlineScale",new Class<?>[]{int.class,int.class,int.class},n,n,n);
  Object ln=construct("org.audiveris.omr.sheet.Scale$LineScale",new Class<?>[]{int.class,int.class,int.class},1,1,1);
  Object beam=construct("org.audiveris.omr.sheet.Scale$BeamScale",new Class<?>[]{int.class,boolean.class},4,false);
  return construct("org.audiveris.omr.sheet.Scale",new Class<?>[]{il.getClass(),ln.getClass(),beam.getClass(),il.getClass(),beam.getClass()},il,ln,beam,null,null);
 }
 static Object makeGlyph(int left,int top,int w,int h,int pattern)throws Exception {
  byte[] pixels=new byte[w*h];Arrays.fill(pixels,(byte)255);
  for(int y=0;y<h;y++)for(int x=0;x<w;x++) {
   boolean ink=switch(pattern){case 0->true;case 1->x==w/2||y==h/2;case 2->x!=w/2||y>=h/3;case 3->x<w/3||x>=2*w/3||y==h/2;case 4->x==0||x==w-1||y==0||y==h-1;case 5->x<w/3||x>=2*w/3;default->Math.pow((x-(w-1)/2.0)/(w/2.0),2)+Math.pow((y-(h-1)/2.0)/(h/2.0),2)<=1;};
   if(ink)pixels[y*w+x]=0;
  }
  Object buffer=construct("ij.process.ByteProcessor",new Class<?>[]{int.class,int.class,byte[].class,java.awt.image.ColorModel.class},w,h,pixels,null);
  Object factory=construct("org.audiveris.omr.run.RunTableFactory",new Class<?>[]{cls("org.audiveris.omr.run.Orientation")},en("org.audiveris.omr.run.Orientation","VERTICAL"));
  Object table=factory.getClass().getMethod("createTable",cls("ij.process.ByteProcessor")).invoke(factory,buffer);
  return construct("org.audiveris.omr.glyph.Glyph",new Class<?>[]{int.class,int.class,cls("org.audiveris.omr.run.RunTable")},left,top,table);
 }
 static String scales(Object s)throws Exception{return "{\"black_head\":"+fields(call(s,"getBlackHeadScale"))+",\"music_font\":"+fields(call(s,"getMusicFontScale"))+"}";}
 public void process(){try {
  Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");
  call(cls("org.audiveris.omr.sheet.grid.GridStep").getConstructor().newInstance(),"doit",sheet);
  Object system=((List<?>)call(sheet,"getSystems")).get(0);java.awt.Rectangle bounds=(java.awt.Rectangle)call(system,"getBounds");
  int left=bounds.x+bounds.width/2,top=bounds.y+10;
  List<String> params=new ArrayList<>(),cases=new ArrayList<>(),measure=new ArrayList<>();
  for(int il=1;il<=128;il++){Object s=scale(il);call(sheet,"setScale",s);Object sizer=construct("org.audiveris.omr.sheet.beam.BlackHeadSizer",new Class<?>[]{cls("org.audiveris.omr.sheet.Sheet")},sheet);params.add("{\"interline\":"+il+",\"parameters\":"+fields(get(sizer,"params"))+"}");}
  for(int il:new int[]{2,3,5,8,11,16,23,32}) {
   Object s=scale(il);call(sheet,"setScale",s);Object sizer=construct("org.audiveris.omr.sheet.beam.BlackHeadSizer",new Class<?>[]{cls("org.audiveris.omr.sheet.Sheet")},sheet),p=get(sizer,"params");
   float radius=(float)((Double)get(p,"diameter")-1)/2;
   Object element=construct("org.audiveris.omr.image.StructureElement",new Class<?>[]{int.class,int.class,float.class,int[].class},0,1,radius,new int[]{0,0});
   Object morph=construct("org.audiveris.omr.image.MorphoProcessor",new Class<?>[]{element.getClass()},element);
   int[][] dims={{il-1,il},{il,il},{il+1,il},{2*il,il},{2*il+1,il},{il,(int)Math.rint(.9*il)-1},{il,(int)Math.rint(.9*il)},{il,(int)Math.rint(1.5*il)},{il,(int)Math.rint(1.5*il)+1},{il,2*il-1},{il,2*il},{il,4*il},{il,4*il+1}};
   for(int[] d:dims)for(int pattern=0;pattern<7;pattern++) {
    int w=Math.max(1,d[0]),h=Math.max(1,d[1]);Object g=makeGlyph(left,top,w,h,pattern);
    String input=canonical(g);boolean before=(Boolean)call(sizer,"checkSpot",g);Object closed=call(sizer,"closeBlackHead",morph,g);
    // Observe process separately on the same input; closeBlackHead's canonical
    // registered return keeps the tagged original object available for inspection.
    if(closed!=null)((Collection<?>)call(closed,"getGroups")).clear();
    call(sizer,"process",new ArrayList<>(List.of(g)));
    cases.add("{\"interline\":"+il+",\"pattern\":"+pattern+",\"input\":"+input+",\"before\":"+before+",\"closed\":"+canonical(closed)+",\"after\":"+(closed==null?"null":call(sizer,"checkSpot",closed))+",\"groups_after_process\":"+(closed==null?"[]":groups(closed))+",\"input_groups_after\":"+groups(g)+",\"registered\":"+(closed!=null&&(Integer)call(closed,"getId")!=0)+"}");
   }
  }
  for(int count:new int[]{0,1,19,20,21,23,24,25,39,40,41}) {
   Object s=scale(16);call(sheet,"setScale",s);Object sizer=construct("org.audiveris.omr.sheet.beam.BlackHeadSizer",new Class<?>[]{cls("org.audiveris.omr.sheet.Sheet")},sheet);
   List<Object> singles=new ArrayList<>();List<String> before=new ArrayList<>();
   for(int i=0;i<count;i++){Object g=makeGlyph(left+i,top,16+(i*7)%9,15+(i*11)%8,0);singles.add(g);before.add(canonical(g));}
   call(sizer,"measureSingles",singles);List<String> sorted=new ArrayList<>();for(Object g:singles)sorted.add(canonical(g));
   measure.add("{\"count\":"+count+",\"input\":["+String.join(",",before)+"],\"sorted\":["+String.join(",",sorted)+"],\"scale\":"+scales(s)+"}");
  }
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"native_outputs_used\":false,\"parameters\":["+String.join(",",params)+"],\"cases\":["+String.join(",",cases)+"],\"measurements\":["+String.join(",",measure)+"]}\n");
 }catch(Exception ex){throw new RuntimeException(ex);}}
}
