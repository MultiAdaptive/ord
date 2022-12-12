use {super::*, transaction_builder::TransactionBuilder};

mod inscribe;
mod inscriptions;
mod receive;
mod satoshis;
mod send;
mod transaction_builder;
mod utxos;

#[derive(Debug, Parser)]
pub(crate) enum Wallet {
  #[clap(about = "List wallet satoshis")]
  Satoshis(satoshis::Satoshis),
  #[clap(about = "Create an inscription")]
  Inscribe(inscribe::Inscribe),
  #[clap(about = "List wallet inscriptions")]
  Inscriptions(inscriptions::Inscriptions),
  #[clap(about = "Generate a receive address")]
  Receive(receive::Receive),
  #[clap(about = "Send a satoshi or inscription")]
  Send(send::Send),
  #[clap(about = "List wallet UTXOs")]
  Utxos(utxos::Utxos),
}

impl Wallet {
  pub(crate) fn run(self, options: Options) -> Result {
    match self {
      Self::Satoshis(satoshis) => satoshis.run(options),
      Self::Inscribe(inscribe) => inscribe.run(options),
      Self::Inscriptions(inscriptions) => inscriptions.run(options),
      Self::Receive(receive) => receive.run(options),
      Self::Send(send) => send.run(options),
      Self::Utxos(utxos) => utxos.run(options),
    }
  }
}

fn list_unspent(options: &Options, index: &Index) -> Result<Vec<(OutPoint, Vec<(u64, u64)>)>> {
  let client = options.bitcoin_rpc_client()?;

  client
    .list_unspent(None, None, None, None, None)?
    .iter()
    .map(|utxo| {
      let outpoint = OutPoint::new(utxo.txid, utxo.vout);
      match index.list(outpoint)? {
        Some(List::Unspent(ordinal_ranges)) => Ok((outpoint, ordinal_ranges)),
        Some(List::Spent) => bail!("output {outpoint} in wallet but is spent according to index"),
        None => bail!("index has not seen {outpoint}"),
      }
    })
    .collect()
}

fn list_utxos(options: &Options) -> Result<BTreeMap<OutPoint, Amount>> {
  let client = options.bitcoin_rpc_client()?;

  Ok(
    client
      .list_unspent(None, None, None, None, None)?
      .iter()
      .map(|utxo| {
        let outpoint = OutPoint::new(utxo.txid, utxo.vout);
        let amount = utxo.amount;

        (outpoint, amount)
      })
      .collect(),
  )
}

fn get_change_addresses(options: &Options, n: usize) -> Result<Vec<Address>> {
  let client = options.bitcoin_rpc_client()?;

  let mut addresses = Vec::new();
  for _ in 0..n {
    addresses.push(
      client
        .call("getrawchangeaddress", &[])
        .context("could not get change addresses from wallet")?,
    );
  }

  Ok(addresses)
}