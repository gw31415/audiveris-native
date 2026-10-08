// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import java.awt.Rectangle;
import java.awt.geom.Line2D;
import java.lang.reflect.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class LedgerReductionProbe extends VoiceLifecycleProbe {
 static Object privateCall(Object o,String name,Object...args)throws Exception{
  for(Method m:o.getClass().getDeclaredMethods())if(m.getName().equals(name)&&m.getParameterCount()==args.length){m.setAccessible(true);return m.invoke(o,args);}throw new NoSuchMethodException(name);
 }
 static Object frame(List<?> staffs,List<Object> nodes,Map<Object,String> labels,Object sig)throws Exception{
  List<Object> maps=new ArrayList<>(),bodies=new ArrayList<>();for(Object s:staffs){List<Object> lines=new ArrayList<>();for(var e:((Map<?,?>)call(s,"getLedgerMap")).entrySet())lines.add(obj("index",e.getKey(),"members",names((List<?>)e.getValue(),labels)));maps.add(obj("staff",call(s,"getId"),"lines",lines));}
  for(Object n:nodes)bodies.add(obj("label",labels.get(n),"removed",call(n,"isRemoved"),"registered",call(sig,"containsVertex",n),"index",call(n,"getIndex"),"owner_staff",call(call(n,"getStaff"),"getId")));
  return obj("maps",maps,"bodies",bodies);
 }
 public static void main(String[]args)throws Exception{
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0);List<?> staffs=(List<?>)call(sys,"getStaves");Object upper=staffs.get(1),lower=staffs.get(2);List<Object> cases=new ArrayList<>();
  for(String mode:List.of("orphan-chain-with-duplicate","head-uses-map-index-and-other-staff","head-boundary-contact","retired-next-reference")){
   for(Object s:staffs)((Map<?,?>)call(s,"getLedgerMap")).clear();Object sig=make("sig.SIGraph",sys),reducer=make("sig.SigReducer",sys,false);Field sf=c("sig.SigReducer").getDeclaredField("sig");sf.setAccessible(true);sf.set(reducer,sig);List<Object> nodes=new ArrayList<>(),inputs=new ArrayList<>(),heads=new ArrayList<>();Map<Object,String> labels=new IdentityHashMap<>();
   for(int i=0;i<(mode.equals("orphan-chain-with-duplicate")?3:mode.equals("retired-next-reference")?2:1);i++){
    Object n=make("sig.inter.LedgerInter",new Line2D.Double(i==2?220:100,200, (i==2?220:100)+11,200),2.,.8);call(n,"setIndex",mode.startsWith("head-")?99:i==1?2:i==2?-1:1);call(n,"setStaff",i==2?lower:upper);call(sig,"addVertex",n);nodes.add(n);labels.put(n,"ledger"+i);
   }
   for(Object s:staffs)((Map<?,?>)call(s,"getLedgerMap")).clear();for(int i=0;i<nodes.size();i++)call(i==2?lower:upper,"addLedger",nodes.get(i),i==2?-1:i+1);
   if(mode.equals("orphan-chain-with-duplicate"))call(upper,"addLedger",nodes.get(0),1);
   if(mode.startsWith("head-")){
    Object h=make("sig.inter.HeadInter",new Rectangle(mode.equals("head-boundary-contact")?111:110,205,8,8),c("glyph.Shape").getField("NOTEHEAD_BLACK").get(null),.8,lower,6.);call(sig,"addVertex",h);heads.add(obj("kind","HeadInter","bounds",rect(call(h,"getBounds")),"pitch",call(h,"getPitch"),"staff_id",call(lower,"getId")));
   }
   if(mode.equals("retired-next-reference")){Object n=nodes.get(1);call(lower,"addLedger",n,-2);call(n,"setStaff",lower);call(n,"remove");}
   for(Object n:nodes)inputs.add(obj("label",labels.get(n),"bounds",rect(call(n,"getBounds")),"index",call(n,"getIndex"),"staff_id",call(call(n,"getStaff"),"getId"),"removed",call(n,"isRemoved")));
   Object before=frame(staffs,nodes,labels,sig);int shared=((Number)privateCall(reducer,"fixAllSharedLedgers")).intValue();int removed=((Number)privateCall(reducer,"checkLedgers")).intValue();Object after=frame(staffs,nodes,labels,sig);int repeat=((Number)privateCall(reducer,"checkLedgers")).intValue();cases.add(obj("name",mode,"nodes",inputs,"heads",heads,"before",before,"shared",shared,"removed_count",removed,"after",after,"repeat_count",repeat));
  }
  Object freshBook=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0])),freshSheet=call(((List<?>)call(freshBook,"getStubs")).get(0),"getSheet");List<Object> real=new ArrayList<>();
  for(Object system:(List<?>)call(freshSheet,"getSystems")){Object sig=call(system,"getSig"),reducer=make("sig.SigReducer",system,false);List<?> staves=(List<?>)call(system,"getStaves");List<Object> nodes=new ArrayList<>();Map<Object,String> labels=new IdentityHashMap<>();for(Object staff:staves)for(Object row:((Map<?,?>)call(staff,"getLedgerMap")).values())for(Object n:(List<?>)row)if(!labels.containsKey(n)){labels.put(n,"ledger"+nodes.size());nodes.add(n);}Object before=frame(staves,nodes,labels,sig);int shared=((Number)privateCall(reducer,"fixAllSharedLedgers")).intValue(),removed=((Number)privateCall(reducer,"checkLedgers")).intValue();real.add(obj("system",call(system,"getId"),"before",before,"shared",shared,"removed_count",removed,"after",frame(staves,nodes,labels,sig)));}
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","original private checkLedgers on declared isolated SIG and actual three Staff objects; original maps, duplicate, distinct body/map indices, other Staff head and retired next-line references","interline",call(call(sheet,"getScale"),"getInterline"),"cases",cases,"real_head_stage",real));System.exit(0);
 }
}
