package examples.gadgets.fibonacci;

import java.math.BigInteger;

import org.bouncycastle.pqc.math.linearalgebra.IntegerFunctions;

import circuit.config.Config;
import circuit.eval.CircuitEvaluator;
import circuit.eval.Instruction;
import circuit.operations.Gadget;
import circuit.structure.ConstantWire;
import circuit.structure.Wire;
import circuit.structure.WireArray;

public class fibonacciVerifyGadget extends Gadget {
	public final static int ITERATIONS = 1 << 20;
	private Wire initVal1;
	private Wire initVal2;
	private Wire outputPublicValue;
	private Wire[] nextVals;
	public fibonacciVerifyGadget(Wire init1, Wire init2) {
		this.initVal1 = init1;
		this.initVal2 = init2;
		this.nextVals = new Wire[ITERATIONS];
		buildCircuit();
	} 

	protected void buildCircuit() {
		nextVals[0] = initVal1;
		nextVals[1] = initVal2;
		for(int i = 2; i<ITERATIONS; i++){
			Wire nextNextVal = nextVals[i-1].mul(nextVals[i-2].mul(20));
			nextVals[i] = nextNextVal;
		
		}
		outputPublicValue = nextVals[ITERATIONS-1];
	}
	
	@Override
	public Wire[] getOutputWires() {
		return new Wire[] { outputPublicValue };
	}

	public Wire getOutputPublicValue() {
		return outputPublicValue;
	}
}



