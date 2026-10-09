// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;
import java.nio.file.*;import java.util.*;import java.awt.Rectangle;
public class RealRepeatDotLookupImageProbe extends RealLedgerMembershipImageProbe {
 private final Object actual;
 public RealRepeatDotLookupImageProbe(Book book,SortedSet<Integer> ids){super(book,ids);actual=book;}
 static Object field(Object o,String name)throws Exception {for(Class<?> k=o.getClass();k!=null;k=k.getSuperclass())try{var f=k.getDeclaredField(name);f.setAccessible(true);return f.get(o);}catch(NoSuchFieldException e){}throw new NoSuchFieldException(name);}
 static String query(Object sys,Object factory,Rectangle bounds,int profile)throws Exception {
 var point=new java.awt.geom.Point2D.Double(bounds.getCenterX(),bounds.getCenterY());Object staff=call(sys,"getClosestStaff",point);Object pp=call(sys,"estimatedPitch",point);boolean valid=(Boolean)c("sig.inter.RepeatDotInter").getMethod("checkPitch",c("sheet.SystemInfo"),java.awt.geom.Point2D.class,int.class).invoke(null,sys,point,profile);
 Object dot=make("sig.inter.RepeatDotInter",null,Double.valueOf(0),staff,pp==null?null:Double.valueOf(((Number)pp).doubleValue()>0?1:-1));call(dot,"setBounds",bounds);Object link=call(dot,"lookupBarLink",sys,call(factory,"getSystemBars"),profile);String result="null";
 if(link!=null){Object bar=field(link,"partner"),r=field(link,"relation");result="{\"bar_bounds\":"+rect(call(bar,"getBounds"))+",\"grade\":"+call(r,"getGrade")+",\"dx\":"+call(r,"getDx")+",\"dy\":"+call(r,"getDy")+",\"source_ratio\":"+call(r,"getSourceRatio")+",\"target_ratio\":"+call(r,"getTargetRatio")+"}";}
 return "{\"bounds\":"+rect(bounds)+",\"profile\":"+profile+",\"staff\":"+(staff==null?"null":call(staff,"getId"))+",\"estimated_pitch\":"+pp+",\"pitch_allowed\":"+valid+",\"bar_lookup\":"+result+"}";
 }
 public void process(){try {
 Object stub=((List<?>)call(actual,"getStubs")).get(0);call(stub,"reachStep",c("step.OmrStep").getField("CHORDS").get(null),false);Object sheet=call(stub,"getSheet");Map<Object,List<?>> optionals=new LinkedHashMap<>();call(make("sheet.symbol.SymbolsFilter",sheet),"process",optionals);List<String> systems=new ArrayList<>();
 for(Object sys:(List<?>)call(sheet,"getSystems")){
 Object factory=make("sheet.symbol.InterFactory",sys),builder=make("sheet.symbol.SymbolsBuilder",sys,factory);call(builder,"buildSymbols",optionals);List<String> candidates=new ArrayList<>(),declared=new ArrayList<>();
 for(Object dot:(List<?>)field(field(factory,"dotFactory"),"dots"))candidates.add(query(sys,factory,(Rectangle)call(dot,"getBounds"),((Number)call(sys,"getProfile")).intValue()));
 for(Object bar:(List<?>)call(factory,"getSystemBars")){Rectangle b=(Rectangle)call(bar,"getBounds");for(int profile=0;profile<=1;profile++)for(int ysign:new int[]{-1,1})for(int xsign:new int[]{-1,1}){int x=(int)Math.rint(b.getCenterX()+xsign*(b.width/2.0+5)),y=(int)Math.rint(b.getCenterY()+ysign*b.height/8.0);declared.add(query(sys,factory,new Rectangle(x-2,y-2,4,4),profile));}}
 systems.add("{\"system\":"+call(sys,"getId")+",\"candidates\":["+String.join(",",candidates)+"],\"declared_bar_neighbors\":["+String.join(",",declared)+"],\"actual_instant_repeat_count\":"+((List<?>)call(call(sys,"getSig"),"inters",c("sig.inter.RepeatDotInter"))).size()+"}");}
 Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"scope\":\"unchanged image original CHORDS then actual SymbolsBuilder DOT candidates plus declared bar-neighbor boxes; repeat pitch and plain bar lookup, no native inputs\",\"systems\":["+String.join(",",systems)+"]}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
