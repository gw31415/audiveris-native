// SPDX-License-Identifier: AGPL-3.0-or-later
// Independent supplied sections; unmodified original BarFilamentFactory/toGlyph.
import java.util.*;import java.nio.file.*;import java.awt.Rectangle;import com.fasterxml.jackson.databind.*;
@SuppressWarnings({"unchecked","rawtypes"})
public class SerifFilamentControlProbe {
 static Object call(Object o,String n,Object...a)throws Exception{return BeamImpactsControlProbe.call(o,n,a);}
 static Object make(String n,Object...a)throws Exception{return BeamImpactsControlProbe.make(n,a);}
 public static void main(String[] args)throws Exception{ObjectMapper mapper=new ObjectMapper();List<Object> results=new ArrayList<>();
 for(JsonNode c:mapper.readTree(Files.readString(Path.of(args[0])))){int il=c.path("interline").asInt(16);Object scale=make("sheet.Scale",make("sheet.Scale$InterlineScale",il,il,il),make("sheet.Scale$LineScale",2,2,2),null,null,null);List<Object> sections=new ArrayList<>();int id=0;JsonNode b=c.get("core");Rectangle core=new Rectangle(b.get(0).asInt(),b.get(1).asInt(),b.get(2).asInt(),b.get(3).asInt());
 for(JsonNode a:c.get("sections")){boolean v=a.get("vertical").asBoolean();Object orient=Enum.valueOf((Class)BeamImpactsControlProbe.cls("run.Orientation"),v?"VERTICAL":"HORIZONTAL"),section=make("lag.DynamicSection",orient);call(section,"setId",++id);call(section,"setFirstPos",a.get("first_pos").asInt());for(JsonNode run:a.get("runs"))call(section,"append",make("run.Run",run.get(0).asInt(),run.get(1).asInt()));Rectangle bounds=(Rectangle)call(section,"getBounds");if(bounds.width<=core.width&&bounds.intersects(core))sections.add(section);}
 sections.sort((a,d)->{try{return Integer.compare(((Rectangle)call(a,"getBounds")).x,((Rectangle)call(d,"getBounds")).x);}catch(Exception e){throw new RuntimeException(e);}});
 Object f=call(make("sheet.grid.BarFilamentFactory",scale),"buildBarFilament",sections,core);Map<String,Object> result=new LinkedHashMap<>();result.put("input",c);result.put("accepted",f!=null);
 if(f!=null){Object glyph=call(f,"toGlyph",(Object)null),table=call(glyph,"getRunTable");Rectangle box=(Rectangle)call(glyph,"getBounds");List<Object> pixels=new ArrayList<>();for(int y=0;y<box.height;y++)for(int x=0;x<box.width;x++)if(((Number)call(table,"get",x,y)).intValue()==0)pixels.add(new int[]{x+box.x,y+box.y});result.put("bounds",new int[]{box.x,box.y,box.width,box.height});result.put("orientation",call(table,"getOrientation").toString());result.put("pixels",pixels);List<Object> members=new ArrayList<>();for(Object m:(Collection<?>)call(f,"getMembers"))members.add(call(m,"getId"));result.put("members",members);}
 results.add(result);}
 mapper.writeValue(Path.of(args[1]).toFile(),Map.of("native_outputs_used",false,"scope","supplied sections; direct original BarFilamentFactory and Filament.toGlyph; not complete-rest conversion","cases",results));}
}
