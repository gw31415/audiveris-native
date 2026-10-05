// SPDX-License-Identifier: AGPL-3.0-or-later
// Test-only read observer after the official, unmodified GRID step.
import org.audiveris.omr.sheet.Book;import java.util.*;import java.nio.file.*;
public class GridFinalProbe extends GridProbe {
 private final Object observed; public GridFinalProbe(Book b,SortedSet<Integer>ids){super(b,ids);observed=b;}
 static String rect(Object raw){if(raw==null)return "null";var r=(java.awt.Rectangle)raw;return "["+r.x+","+r.y+","+r.width+","+r.height+"]";}
 static String ids(Collection<?> objects)throws Exception{List<Object>ids=new ArrayList<>();for(Object o:objects)ids.add(call(o,"getId"));return ids.toString();}
 static String sections(Collection<?>list)throws Exception{List<String>out=new ArrayList<>();for(Object s:list){List<String>runs=new ArrayList<>();for(Object r:(List<?>)call(s,"getRuns"))runs.add("["+call(r,"getStart")+","+call(r,"getLength")+"]");java.awt.Point c=(java.awt.Point)call(s,"getCentroid");out.add("{\"id\":"+call(s,"getId")+",\"first_pos\":"+call(s,"getFirstPos")+",\"centroid\":["+c.x+","+c.y+"],\"bounds\":"+rect(call(s,"getBounds"))+",\"runs\":["+String.join(",",runs)+"]}");}return "["+String.join(",",out)+"]";}
 static String glyph(Object g)throws Exception{Object table=call(g,"getRunTable");List<String>runs=new ArrayList<>();for(int p=0;p<(int)call(table,"getSize");p++){Iterator<?>it=(Iterator<?>)call(table,"iterator",p);while(it.hasNext()){Object r=it.next();runs.add("["+p+","+call(r,"getStart")+","+call(r,"getLength")+"]");}}return "{\"id\":"+call(g,"getId")+",\"bounds\":"+rect(call(g,"getBounds"))+",\"orientation\":\""+call(table,"getOrientation")+"\",\"runs\":["+String.join(",",runs)+"]}";}
 static String noteMask(Object sheet)throws Exception{
  int w=(Integer)call(sheet,"getWidth"),h=(Integer)call(sheet,"getHeight");byte[]mask=new byte[w*h];Arrays.fill(mask,(byte)255);
  Class<?> cls=Class.forName("org.audiveris.omr.sheet.note.DistancesBuilder"),painter=Class.forName("org.audiveris.omr.sheet.note.DistancesBuilder$Painter");
  Object builder=cls.getConstructor(Class.forName("org.audiveris.omr.sheet.Sheet")).newInstance(sheet);
  Object observer=java.lang.reflect.Proxy.newProxyInstance(painter.getClassLoader(),new Class<?>[]{painter},(proxy,method,args)->{
   if(method.getName().equals("paintPixel")){int x=(Integer)args[0],y=(Integer)args[1];mask[y*w+x]=0;}
   else if(method.getName().equals("paintGlyph")){
    Object glyph=args[0],table=call(glyph,"getRunTable");int left=(Integer)call(glyph,"getLeft"),top=(Integer)call(glyph,"getTop");boolean horizontal=call(table,"getOrientation").toString().equals("HORIZONTAL");
    for(int pos=0;pos<(Integer)call(table,"getSize");pos++){Iterator<?>it=(Iterator<?>)call(table,"iterator",pos);while(it.hasNext()){Object run=it.next();int a=(Integer)call(run,"getStart"),len=(Integer)call(run,"getLength");for(int v=a;v<a+len;v++){int x=left+(horizontal?v:pos),y=top+(horizontal?pos:v);mask[y*w+x]=0;}}}
   }return null;
  });
  var paint=cls.getDeclaredMethod("paintLines",painter);paint.setAccessible(true);paint.invoke(builder,observer);
  String mh=HexFormat.of().formatHex(java.security.MessageDigest.getInstance("SHA-256").digest(mask));int count=0;for(byte b:mask)if(b==0)count++;
  Object distances=call(builder,"buildDistances");Class<?>dt=Class.forName("org.audiveris.omr.image.DistanceTable");var get=dt.getMethod("getValue",int.class);var digest=java.security.MessageDigest.getInstance("SHA-256");
  for(int i=0;i<w*h;i++){int v=(Integer)get.invoke(distances,i);digest.update((byte)(v>>>8));digest.update((byte)v);}
  return "{\"mask_sha256\":\""+mh+"\",\"masked_pixel_count\":"+count+",\"distance_sha256_be_i16\":\""+HexFormat.of().formatHex(digest.digest())+"\"}";
 }
 static String headerPreparation(Object sheet)throws Exception{
  List<String>headers=new ArrayList<>();
  for(Object sys:(List<?>)call(sheet,"getSystems")){
   Object builder=Class.forName("org.audiveris.omr.sheet.header.HeaderBuilder").getConstructor(Class.forName("org.audiveris.omr.sheet.SystemInfo")).newInstance(sys);
   Object sig=call(sys,"getSig");Set<Object>before=new HashSet<>((Collection<?>)call(sig,"vertexSet"));call(builder,"computeHeaderStarts");
   for(Object staff:(List<?>)call(sys,"getStaves")){
    Object header=call(staff,"getHeader"),dummy=null;for(Object inter:(Collection<?>)call(sig,"vertexSet"))if(!before.contains(inter)&&call(inter,"getStaff")==staff)dummy=inter;
    Object last=call(staff,"getSideBarline",Class.forName("org.audiveris.omr.util.HorizontalSide").getField("LEFT").get(null));
    if(dummy==null){boolean moved;do{moved=false;List<Object>relations=new ArrayList<>();for(Object rel:(Collection<?>)call(sig,"edgesOf",last))if(rel.getClass().getSimpleName().equals("BarGroupRelation"))relations.add(rel);for(Object rel:relations)if(call(sig,"getEdgeSource",rel)==last){last=call(sig,"getEdgeTarget",rel);moved=true;}}while(moved);}
    String dj=dummy==null?"null":"{\"kind\":\"BarlineInter\",\"shape\":\""+call(dummy,"getShape")+"\",\"staff_id\":"+call(staff,"getId")+",\"bounds\":"+rect(call(dummy,"getBounds"))+",\"grade\":"+call(dummy,"getGrade")+",\"frozen\":"+call(dummy,"isFrozen")+"}";
    headers.add("{\"staff_id\":"+call(staff,"getId")+",\"start\":"+get(header,"start")+",\"stop\":"+get(header,"stop")+",\"last_group_bar_bounds\":"+(dummy==null?rect(call(last,"getBounds")):"null")+",\"dummy_barline\":"+dj+"}");
   }
  }return "["+String.join(",",headers)+"]";
 }
 public void process(){try{
 Object sheet=call(((List<?>)call(observed,"getStubs")).get(0),"getSheet");Object step=Class.forName("org.audiveris.omr.sheet.grid.GridStep").getConstructor().newInstance();call(step,"doit",sheet);List<String>staffs=new ArrayList<>(),systems=new ArrayList<>(),glyphs=new ArrayList<>();
 for(Object staff:(List<?>)call(call(sheet,"getStaffManager"),"getStaves")){List<String>lines=new ArrayList<>();for(Object line:(List<?>)call(staff,"getLines")){Object g=call(line,"getGlyph");lines.add("{\"points\":"+points(get(line,"points"))+",\"thickness\":"+call(line,"getThickness")+",\"glyph_id\":"+call(g,"getId")+"}");}staffs.add("{\"id\":"+call(staff,"getId")+",\"system_id\":"+call(call(staff,"getSystem"),"getId")+",\"lines\":["+String.join(",",lines)+"]}");}
 for(Object sys:(List<?>)call(sheet,"getSystems")){Object sig=call(sys,"getSig");List<String>inters=new ArrayList<>(),rels=new ArrayList<>(),parts=new ArrayList<>(),groups=new ArrayList<>();for(Object inter:(Collection<?>)call(sig,"vertexSet")){Object staff=call(inter,"getStaff"),g=call(inter,"getGlyph");inters.add("{\"id\":"+call(inter,"getId")+",\"kind\":\""+inter.getClass().getSimpleName()+"\",\"shape\":\""+call(inter,"getShape")+"\",\"staff_id\":"+(staff==null?"null":call(staff,"getId"))+",\"glyph_id\":"+(g==null?"null":call(g,"getId"))+",\"bounds\":"+rect(call(inter,"getBounds"))+",\"grade\":"+call(inter,"getGrade")+",\"contextual_grade\":"+call(inter,"getContextualGrade")+",\"frozen\":"+call(inter,"isFrozen")+"}");}
 for(Object rel:(Collection<?>)call(sig,"edgeSet")){rels.add("{\"kind\":\""+rel.getClass().getSimpleName()+"\",\"source\":"+call(call(sig,"getEdgeSource",rel),"getId")+",\"target\":"+call(call(sig,"getEdgeTarget",rel),"getId")+"}");}
 for(Object pg:(List<?>)call(sys,"getPartGroups"))groups.add("{\"number\":"+call(pg,"getNumber")+",\"symbol\":\""+call(pg,"getSymbol")+"\",\"barline\":"+call(pg,"isBarline")+",\"first_staff_id\":"+call(pg,"getFirstStaffId")+",\"last_staff_id\":"+call(pg,"getLastStaffId")+"}");
 for(Object part:(List<?>)call(sys,"getParts"))parts.add("{\"id\":"+call(part,"getId")+",\"staff_ids\":"+ids((Collection<?>)call(part,"getStaves"))+"}");
 systems.add("{\"id\":"+call(sys,"getId")+",\"staff_ids\":"+ids((Collection<?>)call(sys,"getStaves"))+",\"coordinates\":["+call(sys,"getLeft")+","+call(sys,"getTop")+","+call(sys,"getWidth")+","+call(sys,"getBottom")+","+call(sys,"getDeltaY")+"],\"horizontal_section_ids\":"+ids((Collection<?>)call(sys,"getHorizontalSections"))+",\"vertical_section_ids\":"+ids((Collection<?>)call(sys,"getVerticalSections"))+",\"part_groups\":["+String.join(",",groups)+"],\"parts\":["+String.join(",",parts)+"],\"inters\":["+String.join(",",inters)+"],\"relations\":["+String.join(",",rels)+"]}");}
 List<String>pages=new ArrayList<>();for(Object page:(List<?>)call(sheet,"getPages"))pages.add("{\"id\":"+call(page,"getId")+",\"movement_start\":"+call(page,"isMovementStart")+",\"system_ids\":"+ids((Collection<?>)call(page,"getSystems"))+"}");
 for(Object g:(Collection<?>)call(call(sheet,"getGlyphIndex"),"getEntities"))glyphs.add(glyph(g));Object lm=call(sheet,"getLagManager");
 Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"header_preparation\":"+headerPreparation(sheet)+",\"note_preparation\":"+noteMask(sheet)+",\"pages\":["+String.join(",",pages)+"],\"staffs\":["+String.join(",",staffs)+"],\"systems\":["+String.join(",",systems)+"],\"glyphs\":["+String.join(",",glyphs)+"],\"hlag\":"+sections((Collection<?>)call(call(lm,"getLag","hLag"),"getEntities"))+",\"vlag\":"+sections((Collection<?>)call(call(lm,"getLag","vLag"),"getEntities"))+"}");
 }catch(Exception e){throw new RuntimeException(e);}}
}
