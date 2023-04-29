package examples.generators.fibonacci;

import java.util.ArrayList;
import java.math.BigInteger;

import circuit.eval.CircuitEvaluator;
import circuit.structure.CircuitGenerator;
import circuit.structure.Wire;
import circuit.structure.WireArray;

import examples.gadgets.fibonacci.fibonacciVerifyGadget;

public class fibonacciVerifyCircuitGenerator extends CircuitGenerator {
	
	private Wire init1;
	private Wire init2;

	public fibonacciVerifyCircuitGenerator(String circuitName) {
    		super(circuitName);
  	}

	@Override
	protected void buildCircuit() {
		init1 = createInputWire("init1");
		init2 = createInputWire("init2");

		fibonacciVerifyGadget fibonacciGadget = new fibonacciVerifyGadget(init1, init2);
		Wire[] result = fibonacciGadget.getOutputWires();
		makeOutputArray(result, "output of fibonacci");
	}

	@Override
  	public void generateSampleInput(CircuitEvaluator circuitEvaluator) {
		circuitEvaluator.setWireValue(init1, 1);
		circuitEvaluator.setWireValue(init2, 1);
	}

	public static void main(String[] args) throws Exception {
		fibonacciVerifyCircuitGenerator generator;
		generator = new fibonacciVerifyCircuitGenerator("fibonacciexample");
		generator.generateCircuit();
		// generator.printCircuit();
		generator.evalCircuit();
		generator.prepFiles();  // will write circuit to file
		generator.runLibsnark();
		CircuitEvaluator evaluator = generator.getCircuitEvaluator();
		ArrayList<Wire> outputWires = generator.getOutWires();
		if (args.length == 1) {
			System.out.println("OUTPUT OF CIRCUIT: fibonacci verification for (" + args[0] + "," + args[1] + ")*"+ args[2]);
		} else {
			System.out.println("OUTPUT OF CIRCUIT: fibonacci for " + "dummy values:");
		}

		for (int i = 0; i < outputWires.size(); i++){
			Wire out = outputWires.get(i);
				System.out.println(evaluator.getWireValue(out).intValue());
		}
		System.out.println("********************************************************************************");
	}
}
