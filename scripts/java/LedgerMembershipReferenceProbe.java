// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import java.awt.geom.Line2D;
import com.fasterxml.jackson.databind.ObjectMapper;
public class LedgerMembershipReferenceProbe extends VoiceLifecycleProbe {
 static Object frame(Object upper,Object lower,Object ledger,Object sig)throws Exception{
  List<Object> maps=new ArrayList<>();for(Object staff:List.of(upper,lower)){List<Object> lines=new ArrayList<>();for(var e:((Map<?,?>)call(staff,"getLedgerMap")).entrySet()){List<Object> members=new ArrayList<>();for(Object n:(List<?>)e.getValue()){if(n!=ledger)throw new IllegalStateException("unexpected object");members.add(1);}lines.add(obj("index",e.getKey(),"members",members));}maps.add(obj("staff",call(staff,"getId"),"lines",lines));}
  return obj("maps",maps,"index",call(ledger,"getIndex"),"owner_staff",call(call(ledger,"getStaff"),"getId"),"removed",call(ledger,"isRemoved"),"registered",call(sig,"containsVertex",ledger));
 }
 public static void main(String[] args)throws Exception{
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0);Object upper=((List<?>)call(sys,"getStaves")).get(1),lower=((List<?>)call(sys,"getStaves")).get(2);Object sig=make("sig.SIGraph",sys),ledger=make("sig.inter.LedgerInter",new Line2D.Double(350.,573.,365.,573.),2.,.8);call(ledger,"setIndex",1);call(ledger,"setStaff",upper);call(sig,"addVertex",ledger);((Map<?,?>)call(upper,"getLedgerMap")).clear();((Map<?,?>)call(lower,"getLedgerMap")).clear();List<Object> frames=new ArrayList<>();
  call(upper,"addLedger",ledger,1);call(lower,"addLedger",ledger,-6);call(ledger,"setStaff",lower);frames.add(obj("phase","shared-last-staff","state",frame(upper,lower,ledger,sig)));
  call(upper,"addLedger",ledger,1);frames.add(obj("phase","duplicate-same-reference","state",frame(upper,lower,ledger,sig)));
  call(ledger,"remove");frames.add(obj("phase","automatic-remove-last-staff-only","state",frame(upper,lower,ledger,sig)));
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","declared original same Ledger object in two actual Staff maps with different indices, repeated same reference, last Staff assignment and automatic removal; not real image shared-ledger admission","frames",frames));System.exit(0);
 }
}
