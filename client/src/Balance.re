[@bs.module] external css : Js.t({..}) = "./Balance.css";

let component = ReasonReact.statelessComponent(__MODULE__);

let make = (~message, _children) => {
  ...component,
  render: (_) =>
    <div className=css##balance>
      (message |> ReasonReact.stringToElement)
    </div>,
};

